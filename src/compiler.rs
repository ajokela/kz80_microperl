//! Bytecode compiler for MicroPerl

use std::collections::HashMap;

use crate::ast::{BinOp, Expr, Program, Stmt, UnaryOp};
use crate::bytecode::{Module, Op};

/// Compiler state
pub struct Compiler {
    module: Module,

    /// Global variables: name -> index
    globals: HashMap<String, u16>,

    /// Local variables in current scope: name -> stack offset
    locals: Vec<HashMap<String, u8>>,

    /// Subroutine addresses: name -> (address, num_params)
    subs: HashMap<String, (u16, u8)>,

    /// Loop context for last/next: (continue_addr, break_addr)
    loop_stack: Vec<(u16, Vec<usize>)>,

    /// Forward references to patch
    forward_refs: Vec<(String, usize)>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            module: Module::new(),
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            subs: HashMap::new(),
            loop_stack: Vec::new(),
            forward_refs: Vec::new(),
        }
    }

    pub fn compile(mut self, program: &Program) -> Result<Module, String> {
        // First pass: collect subroutine declarations
        for stmt in &program.statements {
            if let Stmt::Sub { name, params, .. } = stmt {
                self.subs.insert(name.clone(), (0, params.len() as u8));
            }
        }

        // Compile main code
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }

        // Add halt at end
        self.module.emit(Op::Halt);

        // Patch forward references
        for (name, patch_pos) in &self.forward_refs {
            if let Some((addr, _)) = self.subs.get(name) {
                self.module.patch_addr(*patch_pos, *addr);
            } else {
                return Err(format!("Undefined subroutine: {}", name));
            }
        }

        // Copy sub info to module
        for (name, (addr, params)) in &self.subs {
            self.module.subs.push((name.clone(), *addr, *params));
        }

        Ok(self.module)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.module.emit(Op::Pop); // Discard result
            }

            Stmt::My(vars, init) => {
                // Allocate local variables
                for var in vars {
                    let idx = self.locals.last().unwrap().len() as u8;
                    self.locals.last_mut().unwrap().insert(var.clone(), idx);
                }

                // Initialize if provided
                if let Some(init_expr) = init {
                    if vars.len() == 1 {
                        self.compile_expr(init_expr)?;
                        let idx = *self.locals.last().unwrap().get(&vars[0]).unwrap();
                        self.module.emit_byte(Op::StoreLocal, idx);
                    } else {
                        // List assignment - compile expr and distribute
                        self.compile_expr(init_expr)?;
                        for (i, var) in vars.iter().enumerate() {
                            if i < vars.len() - 1 {
                                self.module.emit(Op::Dup);
                            }
                            self.module.emit_word(Op::Push, i as u16);
                            self.module.emit(Op::ArrGet);
                            let idx = *self.locals.last().unwrap().get(var).unwrap();
                            self.module.emit_byte(Op::StoreLocal, idx);
                        }
                    }
                }
            }

            Stmt::Our(vars, init) => {
                // Allocate global variables
                for var in vars {
                    let idx = self.globals.len() as u16;
                    self.globals.insert(var.clone(), idx);
                    self.module.globals.push(var.clone());
                }

                if let Some(init_expr) = init {
                    if vars.len() == 1 {
                        self.compile_expr(init_expr)?;
                        let idx = *self.globals.get(&vars[0]).unwrap();
                        self.module.emit_word(Op::StoreGlobal, idx);
                    }
                }
            }

            Stmt::If { cond, then_block, elsif_blocks, else_block } => {
                self.compile_expr(cond)?;

                // Jump to elsif/else if false
                let jump_pos = self.module.pos() as usize + 1;
                self.module.emit_word(Op::JumpIfNot, 0); // Placeholder

                // Then block
                for s in then_block {
                    self.compile_stmt(s)?;
                }

                // Jump over else blocks
                let mut end_jumps = vec![];
                if !elsif_blocks.is_empty() || else_block.is_some() {
                    end_jumps.push(self.module.pos() as usize + 1);
                    self.module.emit_word(Op::Jump, 0);
                }

                // Patch jump to here
                self.module.patch_addr(jump_pos, self.module.pos());

                // Elsif blocks
                for (elsif_cond, elsif_body) in elsif_blocks {
                    self.compile_expr(elsif_cond)?;
                    let elsif_jump = self.module.pos() as usize + 1;
                    self.module.emit_word(Op::JumpIfNot, 0);

                    for s in elsif_body {
                        self.compile_stmt(s)?;
                    }

                    end_jumps.push(self.module.pos() as usize + 1);
                    self.module.emit_word(Op::Jump, 0);

                    self.module.patch_addr(elsif_jump, self.module.pos());
                }

                // Else block
                if let Some(else_body) = else_block {
                    for s in else_body {
                        self.compile_stmt(s)?;
                    }
                }

                // Patch all end jumps
                let end_pos = self.module.pos();
                for jump_pos in end_jumps {
                    self.module.patch_addr(jump_pos, end_pos);
                }
            }

            Stmt::Unless { cond, then_block, else_block } => {
                self.compile_expr(cond)?;

                let jump_pos = self.module.pos() as usize + 1;
                self.module.emit_word(Op::JumpIf, 0); // Jump if TRUE (opposite of if)

                for s in then_block {
                    self.compile_stmt(s)?;
                }

                if let Some(else_body) = else_block {
                    let end_jump = self.module.pos() as usize + 1;
                    self.module.emit_word(Op::Jump, 0);
                    self.module.patch_addr(jump_pos, self.module.pos());

                    for s in else_body {
                        self.compile_stmt(s)?;
                    }

                    self.module.patch_addr(end_jump, self.module.pos());
                } else {
                    self.module.patch_addr(jump_pos, self.module.pos());
                }
            }

            Stmt::While { cond, body } => {
                let loop_start = self.module.pos();
                self.loop_stack.push((loop_start, vec![]));

                self.compile_expr(cond)?;
                let exit_jump = self.module.pos() as usize + 1;
                self.module.emit_word(Op::JumpIfNot, 0);

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.module.emit_word(Op::Jump, loop_start);

                let end_pos = self.module.pos();
                self.module.patch_addr(exit_jump, end_pos);

                // Patch break jumps
                let (_, break_jumps) = self.loop_stack.pop().unwrap();
                for pos in break_jumps {
                    self.module.patch_addr(pos, end_pos);
                }
            }

            Stmt::Until { cond, body } => {
                let loop_start = self.module.pos();
                self.loop_stack.push((loop_start, vec![]));

                self.compile_expr(cond)?;
                let exit_jump = self.module.pos() as usize + 1;
                self.module.emit_word(Op::JumpIf, 0); // Exit if TRUE

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.module.emit_word(Op::Jump, loop_start);

                let end_pos = self.module.pos();
                self.module.patch_addr(exit_jump, end_pos);

                let (_, break_jumps) = self.loop_stack.pop().unwrap();
                for pos in break_jumps {
                    self.module.patch_addr(pos, end_pos);
                }
            }

            Stmt::For { init, cond, step, body } => {
                // New scope for loop variable
                self.locals.push(HashMap::new());

                if let Some(init_stmt) = init {
                    self.compile_stmt(init_stmt)?;
                }

                let loop_start = self.module.pos();
                let continue_pos = loop_start; // For 'next'
                self.loop_stack.push((continue_pos, vec![]));

                let exit_jump = if let Some(cond_expr) = cond {
                    self.compile_expr(cond_expr)?;
                    let pos = self.module.pos() as usize + 1;
                    self.module.emit_word(Op::JumpIfNot, 0);
                    Some(pos)
                } else {
                    None
                };

                for s in body {
                    self.compile_stmt(s)?;
                }

                // Step expression
                if let Some(step_expr) = step {
                    self.compile_expr(step_expr)?;
                    self.module.emit(Op::Pop);
                }

                self.module.emit_word(Op::Jump, loop_start);

                let end_pos = self.module.pos();
                if let Some(jump_pos) = exit_jump {
                    self.module.patch_addr(jump_pos, end_pos);
                }

                let (_, break_jumps) = self.loop_stack.pop().unwrap();
                for pos in break_jumps {
                    self.module.patch_addr(pos, end_pos);
                }

                self.locals.pop();
            }

            Stmt::Foreach { var, list, body } => {
                self.locals.push(HashMap::new());

                // Allocate loop variable
                let var_idx = 0u8;
                self.locals.last_mut().unwrap().insert(var.clone(), var_idx);

                // Compile list and get iterator index
                self.compile_expr(list)?;
                self.module.emit_word(Op::Push, 0); // Index = 0

                let loop_start = self.module.pos();
                self.loop_stack.push((loop_start, vec![]));

                // Check if index < array length
                self.module.emit(Op::Over);  // [arr, idx, arr]
                self.module.emit(Op::ArrLen); // [arr, idx, len]
                self.module.emit(Op::Over);  // [arr, idx, len, idx]
                self.module.emit(Op::CmpLt); // [arr, idx, idx<len]

                let exit_jump = self.module.pos() as usize + 1;
                self.module.emit_word(Op::JumpIfNot, 0);

                // Get current element
                self.module.emit(Op::Over);  // [arr, idx, arr]
                self.module.emit(Op::Over);  // [arr, idx, arr, idx]
                self.module.emit(Op::ArrGet); // [arr, idx, elem]
                self.module.emit_byte(Op::StoreLocal, var_idx);

                for s in body {
                    self.compile_stmt(s)?;
                }

                // Increment index
                self.module.emit(Op::Inc);
                self.module.emit_word(Op::Jump, loop_start);

                let end_pos = self.module.pos();
                self.module.patch_addr(exit_jump, end_pos);

                // Clean up stack
                self.module.emit(Op::Pop); // Pop index
                self.module.emit(Op::Pop); // Pop array

                let (_, break_jumps) = self.loop_stack.pop().unwrap();
                for pos in break_jumps {
                    self.module.patch_addr(pos, end_pos);
                }

                self.locals.pop();
            }

            Stmt::Last => {
                if let Some((_, ref mut break_jumps)) = self.loop_stack.last_mut() {
                    break_jumps.push(self.module.pos() as usize + 1);
                    self.module.emit_word(Op::Jump, 0);
                } else {
                    return Err("'last' outside of loop".to_string());
                }
            }

            Stmt::Next => {
                if let Some((continue_pos, _)) = self.loop_stack.last() {
                    self.module.emit_word(Op::Jump, *continue_pos);
                } else {
                    return Err("'next' outside of loop".to_string());
                }
            }

            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e)?;
                    self.module.emit(Op::ReturnVal);
                } else {
                    self.module.emit(Op::Return);
                }
            }

            Stmt::Sub { name, params, body } => {
                // Jump over subroutine body
                let skip_jump = self.module.pos() as usize + 1;
                self.module.emit_word(Op::Jump, 0);

                // Record subroutine address
                let sub_addr = self.module.pos();
                self.subs.insert(name.clone(), (sub_addr, params.len() as u8));

                // Set up frame
                self.locals.push(HashMap::new());
                self.module.emit_byte(Op::EnterFrame, params.len() as u8);

                // Parameters are already on stack, map them to locals
                for (i, param) in params.iter().enumerate() {
                    self.locals.last_mut().unwrap().insert(param.clone(), i as u8);
                }

                // Compile body
                for s in body {
                    self.compile_stmt(s)?;
                }

                // Default return
                self.module.emit(Op::LeaveFrame);
                self.module.emit(Op::Return);

                self.locals.pop();

                // Patch skip jump
                self.module.patch_addr(skip_jump, self.module.pos());
            }

            Stmt::Print(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr)?;
                    self.module.emit(Op::Print);
                }
            }

            Stmt::Say(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr)?;
                    self.module.emit(Op::Print);
                }
                self.module.emit(Op::PrintLn);
            }

            Stmt::Block(stmts) => {
                self.locals.push(HashMap::new());
                for s in stmts {
                    self.compile_stmt(s)?;
                }
                self.locals.pop();
            }

            Stmt::Use(_) | Stmt::Package(_) => {
                // Ignored for now
            }
        }

        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Integer(n) => {
                self.module.emit_word(Op::Push, *n as u16);
            }

            Expr::Float(f) => {
                // Convert to fixed point or truncate
                self.module.emit_word(Op::Push, *f as i32 as u16);
            }

            Expr::String(s) => {
                let idx = self.module.add_string(s);
                self.module.emit_word(Op::PushStr, idx);
            }

            Expr::ScalarVar(name) => {
                if let Some(idx) = self.find_local(name) {
                    self.module.emit_byte(Op::LoadLocal, idx);
                } else if let Some(idx) = self.globals.get(name) {
                    self.module.emit_word(Op::LoadGlobal, *idx);
                } else {
                    return Err(format!("Undefined variable: ${}", name));
                }
            }

            Expr::ArrayVar(name) => {
                if let Some(idx) = self.find_local(name) {
                    self.module.emit_byte(Op::LoadLocal, idx);
                } else if let Some(idx) = self.globals.get(name) {
                    self.module.emit_word(Op::LoadGlobal, *idx);
                } else {
                    return Err(format!("Undefined array: @{}", name));
                }
            }

            Expr::HashVar(name) => {
                if let Some(idx) = self.find_local(name) {
                    self.module.emit_byte(Op::LoadLocal, idx);
                } else if let Some(idx) = self.globals.get(name) {
                    self.module.emit_word(Op::LoadGlobal, *idx);
                } else {
                    return Err(format!("Undefined hash: %{}", name));
                }
            }

            Expr::ArrayIndex(arr, idx) => {
                self.compile_expr(arr)?;
                self.compile_expr(idx)?;
                self.module.emit(Op::ArrGet);
            }

            Expr::HashIndex(hash, key) => {
                self.compile_expr(hash)?;
                self.compile_expr(key)?;
                self.module.emit(Op::HashGet);
            }

            Expr::BinOp(left, op, right) => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                let opcode = match op {
                    BinOp::Add => Op::Add,
                    BinOp::Sub => Op::Sub,
                    BinOp::Mul => Op::Mul,
                    BinOp::Div => Op::Div,
                    BinOp::Mod => Op::Mod,
                    BinOp::Concat => Op::StrCat,
                    BinOp::Eq => Op::CmpEq,
                    BinOp::Ne => Op::CmpNe,
                    BinOp::Lt => Op::CmpLt,
                    BinOp::Gt => Op::CmpGt,
                    BinOp::Le => Op::CmpLe,
                    BinOp::Ge => Op::CmpGe,
                    BinOp::Cmp => Op::Cmp,
                    BinOp::StrEq => Op::StrEq,
                    BinOp::StrNe => Op::StrNe,
                    BinOp::StrLt => Op::StrLt,
                    BinOp::StrGt => Op::StrGt,
                    BinOp::StrLe => Op::StrLe,
                    BinOp::StrGe => Op::StrGe,
                    BinOp::StrCmp => Op::StrCmp,
                    BinOp::And => Op::And,
                    BinOp::Or => Op::Or,
                    BinOp::BitAnd => Op::BitAnd,
                    BinOp::BitOr => Op::BitOr,
                    BinOp::BitXor => Op::BitXor,
                    BinOp::ShiftLeft => Op::Shl,
                    BinOp::ShiftRight => Op::Shr,
                    BinOp::Pow => {
                        // No native pow, would need runtime function
                        return Err("Power operator not yet implemented".to_string());
                    }
                };
                self.module.emit(opcode);
            }

            Expr::UnaryOp(op, expr) => {
                self.compile_expr(expr)?;
                match op {
                    UnaryOp::Neg => self.module.emit(Op::Neg),
                    UnaryOp::Not => self.module.emit(Op::Not),
                    UnaryOp::BitNot => self.module.emit(Op::BitNot),
                    UnaryOp::Ref => {
                        return Err("References not yet implemented".to_string());
                    }
                }
            }

            Expr::PreIncrement(expr) => {
                self.compile_lvalue_addr(expr)?;
                self.module.emit(Op::Dup);
                self.compile_load_indirect(expr)?;
                self.module.emit(Op::Inc);
                self.module.emit(Op::Dup);
                self.compile_store_indirect(expr)?;
            }

            Expr::PreDecrement(expr) => {
                self.compile_lvalue_addr(expr)?;
                self.module.emit(Op::Dup);
                self.compile_load_indirect(expr)?;
                self.module.emit(Op::Dec);
                self.module.emit(Op::Dup);
                self.compile_store_indirect(expr)?;
            }

            Expr::PostIncrement(expr) => {
                self.compile_expr(expr)?;
                self.module.emit(Op::Dup);
                self.module.emit(Op::Inc);
                self.compile_assign_expr(expr)?;
            }

            Expr::PostDecrement(expr) => {
                self.compile_expr(expr)?;
                self.module.emit(Op::Dup);
                self.module.emit(Op::Dec);
                self.compile_assign_expr(expr)?;
            }

            Expr::Assign(target, value) => {
                self.compile_expr(value)?;
                self.module.emit(Op::Dup); // Keep value on stack as result
                self.compile_assign_expr(target)?;
            }

            Expr::OpAssign(target, op, value) => {
                self.compile_expr(target)?;
                self.compile_expr(value)?;

                let opcode = match op {
                    BinOp::Add => Op::Add,
                    BinOp::Sub => Op::Sub,
                    BinOp::Mul => Op::Mul,
                    BinOp::Div => Op::Div,
                    BinOp::Concat => Op::StrCat,
                    _ => return Err(format!("Unsupported op-assign: {:?}", op)),
                };
                self.module.emit(opcode);
                self.module.emit(Op::Dup);
                self.compile_assign_expr(target)?;
            }

            Expr::Call(name, args) => {
                // Push arguments
                for arg in args {
                    self.compile_expr(arg)?;
                }

                if let Some((addr, _)) = self.subs.get(name) {
                    self.module.emit_word(Op::Call, *addr);
                } else {
                    // Forward reference
                    self.forward_refs.push((name.clone(), self.module.pos() as usize + 1));
                    self.module.emit_word(Op::Call, 0);
                }
            }

            Expr::MethodCall(obj, method, args) => {
                self.compile_expr(obj)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                // Would need runtime method dispatch
                return Err(format!("Method calls not yet implemented: {}", method));
            }

            Expr::List(items) => {
                self.module.emit_byte(Op::NewArray, items.len() as u8);
                for (i, item) in items.iter().enumerate() {
                    self.module.emit(Op::Dup);
                    self.module.emit_word(Op::Push, i as u16);
                    self.compile_expr(item)?;
                    self.module.emit(Op::ArrSet);
                }
            }

            Expr::Hash(pairs) => {
                self.module.emit(Op::NewHash);
                for (key, value) in pairs {
                    self.module.emit(Op::Dup);
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                    self.module.emit(Op::HashSet);
                }
            }

            Expr::Ternary(cond, then_expr, else_expr) => {
                self.compile_expr(cond)?;
                let else_jump = self.module.pos() as usize + 1;
                self.module.emit_word(Op::JumpIfNot, 0);

                self.compile_expr(then_expr)?;
                let end_jump = self.module.pos() as usize + 1;
                self.module.emit_word(Op::Jump, 0);

                self.module.patch_addr(else_jump, self.module.pos());
                self.compile_expr(else_expr)?;

                self.module.patch_addr(end_jump, self.module.pos());
            }

            Expr::Range(_, _) => {
                return Err("Range expressions not yet implemented".to_string());
            }

            Expr::Match(expr, pattern, _flags) => {
                // Compile the string to match
                self.compile_expr(expr)?;
                // Push the regex pattern as a string
                let idx = self.module.add_string(pattern);
                self.module.emit_word(Op::PushStr, idx);
                // Emit match opcode
                self.module.emit(Op::Match);
            }

            Expr::NotMatch(expr, pattern, _flags) => {
                // Compile the string to match
                self.compile_expr(expr)?;
                // Push the regex pattern as a string
                let idx = self.module.add_string(pattern);
                self.module.emit_word(Op::PushStr, idx);
                // Emit match opcode then negate
                self.module.emit(Op::Match);
                self.module.emit(Op::Not);
            }

            Expr::Ref(expr) => {
                return Err("References not yet implemented".to_string());
            }

            Expr::Deref(expr) => {
                return Err("Dereferences not yet implemented".to_string());
            }
        }

        Ok(())
    }

    fn compile_assign_expr(&mut self, target: &Expr) -> Result<(), String> {
        match target {
            Expr::ScalarVar(name) => {
                if let Some(idx) = self.find_local(name) {
                    self.module.emit_byte(Op::StoreLocal, idx);
                } else if let Some(idx) = self.globals.get(name) {
                    self.module.emit_word(Op::StoreGlobal, *idx);
                } else {
                    // Auto-vivify as local
                    let idx = self.locals.last().unwrap().len() as u8;
                    self.locals.last_mut().unwrap().insert(name.clone(), idx);
                    self.module.emit_byte(Op::StoreLocal, idx);
                }
            }
            Expr::ArrayIndex(arr, idx) => {
                // Stack: [value, arr, idx]
                self.compile_expr(arr)?;
                self.compile_expr(idx)?;
                self.module.emit(Op::ArrSet);
            }
            Expr::HashIndex(hash, key) => {
                self.compile_expr(hash)?;
                self.compile_expr(key)?;
                self.module.emit(Op::HashSet);
            }
            _ => return Err("Invalid assignment target".to_string()),
        }
        Ok(())
    }

    fn compile_lvalue_addr(&mut self, _expr: &Expr) -> Result<(), String> {
        // For pre-increment/decrement - simplified
        Ok(())
    }

    fn compile_load_indirect(&mut self, expr: &Expr) -> Result<(), String> {
        self.compile_expr(expr)
    }

    fn compile_store_indirect(&mut self, expr: &Expr) -> Result<(), String> {
        self.compile_assign_expr(expr)
    }

    fn find_local(&self, name: &str) -> Option<u8> {
        for scope in self.locals.iter().rev() {
            if let Some(idx) = scope.get(name) {
                return Some(*idx);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::bytecode::Op;

    fn compile(code: &str) -> Result<Module, String> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        let compiler = Compiler::new();
        compiler.compile(&program)
    }

    fn get_opcodes(module: &Module) -> Vec<Op> {
        let mut ops = Vec::new();
        let mut pc = 0;
        while pc < module.code.len() {
            let op = Op::from_byte(module.code[pc]);
            ops.push(op);
            pc += op.size();
        }
        ops
    }

    // === Match expression compilation tests ===

    #[test]
    fn test_compile_match_simple() {
        let module = compile("my $x = \"test\"; $x =~ /hello/;").unwrap();
        let ops = get_opcodes(&module);

        // Should have: LoadLocal, PushStr, Match, Pop, Halt
        assert!(ops.contains(&Op::Match), "Should contain Match opcode");
    }

    #[test]
    fn test_compile_not_match_simple() {
        let module = compile("my $x = \"test\"; $x !~ /hello/;").unwrap();
        let ops = get_opcodes(&module);

        // Should have: LoadLocal, PushStr, Match, Not, Pop, Halt
        assert!(ops.contains(&Op::Match), "Should contain Match opcode");
        assert!(ops.contains(&Op::Not), "Should contain Not opcode for negation");
    }

    #[test]
    fn test_compile_match_stores_pattern_string() {
        let module = compile(r#"my $x = "test"; $x =~ /test_pattern/;"#).unwrap();

        // Pattern should be in string table
        assert!(module.strings.contains(&"test_pattern".to_string()),
                "Pattern should be in string table");
    }

    #[test]
    fn test_compile_match_in_if() {
        let module = compile(r#"my $x = "test"; if ($x =~ /yes/) { print "ok"; }"#).unwrap();
        let ops = get_opcodes(&module);

        // Should have Match and JumpIfNot for the if condition
        assert!(ops.contains(&Op::Match));
        assert!(ops.contains(&Op::JumpIfNot));
    }

    #[test]
    fn test_compile_not_match_in_if() {
        let module = compile(r#"my $x = "test"; if ($x !~ /bad/) { print "ok"; }"#).unwrap();
        let ops = get_opcodes(&module);

        // Should have Match, Not, and JumpIfNot
        assert!(ops.contains(&Op::Match));
        assert!(ops.contains(&Op::Not));
        assert!(ops.contains(&Op::JumpIfNot));
    }

    #[test]
    fn test_compile_match_preserves_wildcard() {
        let module = compile(r#"my $x = "hello"; $x =~ /h.llo/;"#).unwrap();

        // Pattern with wildcard should be stored literally
        assert!(module.strings.contains(&"h.llo".to_string()),
                "Wildcard pattern should be preserved");
    }

    #[test]
    fn test_compile_match_empty_pattern() {
        let module = compile(r#"my $x = "test"; $x =~ //;"#).unwrap();

        // Empty pattern should work
        assert!(module.strings.contains(&"".to_string()),
                "Empty pattern should be in string table");
    }

    #[test]
    fn test_compile_multiple_matches() {
        let module = compile(r#"
            my $a = "one";
            my $b = "two";
            my $c = "three";
            $a =~ /one/;
            $b =~ /two/;
            $c !~ /three/;
        "#).unwrap();

        // All three patterns should be in string table
        assert!(module.strings.contains(&"one".to_string()));
        assert!(module.strings.contains(&"two".to_string()));
        assert!(module.strings.contains(&"three".to_string()));

        // Should have multiple Match opcodes
        let ops = get_opcodes(&module);
        let match_count = ops.iter().filter(|op| **op == Op::Match).count();
        assert_eq!(match_count, 3, "Should have 3 Match opcodes");
    }

    #[test]
    fn test_compile_match_with_and() {
        let module = compile(r#"my $a = "x"; my $b = "y"; if ($a =~ /x/ && $b =~ /y/) { print 1; }"#).unwrap();
        let ops = get_opcodes(&module);

        // Two Match opcodes for the two regex matches
        let match_count = ops.iter().filter(|op| **op == Op::Match).count();
        assert_eq!(match_count, 2, "Should have 2 Match opcodes");
    }

    #[test]
    fn test_compile_match_bytecode_sequence() {
        let module = compile(r#"my $x = "hello"; $x =~ /ell/;"#).unwrap();

        // Verify the bytecode sequence for the match
        // Find the Match opcode and verify PushStr comes before it
        let mut found_pushstr = false;
        let mut found_match_after_pushstr = false;
        let mut pc = 0;

        while pc < module.code.len() {
            let op = Op::from_byte(module.code[pc]);
            if op == Op::PushStr {
                found_pushstr = true;
            }
            if op == Op::Match && found_pushstr {
                found_match_after_pushstr = true;
            }
            pc += op.size();
        }

        assert!(found_match_after_pushstr,
                "Match should come after PushStr for the pattern");
    }

    #[test]
    fn test_compile_not_match_bytecode_sequence() {
        let module = compile(r#"my $x = "hello"; $x !~ /bad/;"#).unwrap();

        // For NotMatch, Not should come immediately after Match
        let ops = get_opcodes(&module);

        // Find Match and verify Not follows
        for (i, op) in ops.iter().enumerate() {
            if *op == Op::Match && i + 1 < ops.len() {
                assert_eq!(ops[i + 1], Op::Not,
                          "Not should immediately follow Match for !~");
                return;
            }
        }
        panic!("Should have found Match followed by Not");
    }

    // === Edge case tests ===

    #[test]
    fn test_compile_match_special_chars_in_pattern() {
        let module = compile(r#"my $x = "test123"; $x =~ /\d+\s*/;"#).unwrap();

        // Pattern should be stored with escapes preserved
        assert!(module.strings.contains(&r"\d+\s*".to_string()));
    }

    #[test]
    fn test_compile_match_on_string_literal() {
        let module = compile(r#""hello world" =~ /world/;"#).unwrap();

        // Should compile without error
        assert!(module.strings.contains(&"hello world".to_string()));
        assert!(module.strings.contains(&"world".to_string()));
    }

    #[test]
    fn test_compile_while_with_match() {
        let module = compile(r#"my $line = "data"; while ($line =~ /data/) { print $line; }"#).unwrap();
        let ops = get_opcodes(&module);

        // Should have Match in the loop condition
        assert!(ops.contains(&Op::Match));
        assert!(ops.contains(&Op::Jump), "While loop should have Jump for looping");
    }
}
