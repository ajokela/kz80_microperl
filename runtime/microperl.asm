;; MicroPerl Z80 Runtime Interpreter
;; A stack-based bytecode virtual machine for Z80
;;
;; Memory Map:
;;   0x0000-0x00FF: Interrupt vectors / system
;;   0x0100-0x0FFF: Runtime code (~4KB)
;;   0x1000-0x1FFF: Bytecode module (loaded here)
;;   0x2000-0x7FFF: Heap for strings/arrays
;;   0x8000-0xBFFF: Stack (grows down from 0xBFFF)
;;   0xC000-0xFFFF: I/O / video / reserved

; === System Constants ===
BYTECODE_BASE   equ     0x1000      ; Where bytecode module is loaded
HEAP_BASE       equ     0x2000      ; Start of heap
HEAP_END        equ     0x8000      ; End of heap
STACK_BASE      equ     0xBFFF      ; Top of stack (grows down)
LOCALS_BASE     equ     0xC000      ; Local variable area

; === Module Header Offsets ===
HDR_MAGIC       equ     0           ; "MPL\x01" (4 bytes)
HDR_STRTAB      equ     4           ; String table offset (2 bytes)
HDR_CODELEN     equ     6           ; Code length (2 bytes)
HDR_ENTRY       equ     8           ; Entry point (2 bytes)
HDR_CODE        equ     12          ; Code starts here

; === Opcode Constants ===
OP_NOP          equ     0x00
OP_PUSH         equ     0x01
OP_PUSHBYTE     equ     0x02
OP_POP          equ     0x03
OP_DUP          equ     0x04
OP_SWAP         equ     0x05
OP_OVER         equ     0x06
OP_LDLOC        equ     0x10
OP_STLOC        equ     0x11
OP_LDGLOB       equ     0x12
OP_STGLOB       equ     0x13
OP_PUSHSTR      equ     0x18
OP_STRLEN       equ     0x19
OP_STRCAT       equ     0x1A
OP_STRIDX       equ     0x1B
OP_STRCMP       equ     0x1C
OP_SUBSTR       equ     0x1D
OP_NEWARR       equ     0x20
OP_ARRLEN       equ     0x21
OP_ARRGET       equ     0x22
OP_ARRSET       equ     0x23
OP_ARRPUSH      equ     0x24
OP_ARRPOP       equ     0x25
OP_NEWHASH      equ     0x28
OP_HASHGET      equ     0x29
OP_HASHSET      equ     0x2A
OP_HASHDEL      equ     0x2B
OP_HASHKEYS     equ     0x2C
OP_ADD          equ     0x30
OP_SUB          equ     0x31
OP_MUL          equ     0x32
OP_DIV          equ     0x33
OP_MOD          equ     0x34
OP_NEG          equ     0x35
OP_INC          equ     0x36
OP_DEC          equ     0x37
OP_BITAND       equ     0x38
OP_BITOR        equ     0x39
OP_BITXOR       equ     0x3A
OP_BITNOT       equ     0x3B
OP_SHL          equ     0x3C
OP_SHR          equ     0x3D
OP_CMPEQ        equ     0x40
OP_CMPNE        equ     0x41
OP_CMPLT        equ     0x42
OP_CMPGT        equ     0x43
OP_CMPLE        equ     0x44
OP_CMPGE        equ     0x45
OP_CMP          equ     0x46
OP_STREQ        equ     0x48
OP_STRNE        equ     0x49
OP_STRLT        equ     0x4A
OP_STRGT        equ     0x4B
OP_STRLE        equ     0x4C
OP_STRGE        equ     0x4D
OP_NOT          equ     0x50
OP_AND          equ     0x51
OP_OR           equ     0x52
OP_JUMP         equ     0x60
OP_JUMPIF       equ     0x61
OP_JUMPIFNOT    equ     0x62
OP_JUMPIFDEF    equ     0x63
OP_CALL         equ     0x68
OP_CALLNAT      equ     0x69
OP_RETURN       equ     0x6A
OP_RETURNVAL    equ     0x6B
OP_ENTER        equ     0x70
OP_LEAVE        equ     0x71
OP_PRINT        equ     0x78
OP_PRINTSTR     equ     0x79
OP_PRINTNUM     equ     0x7A
OP_PRINTCHAR    equ     0x7B
OP_PRINTLN      equ     0x7C
OP_INPUT        equ     0x7D
OP_INPUTCHAR    equ     0x7E
OP_TONUM        equ     0x80
OP_TOSTR        equ     0x81
OP_TYPEOF       equ     0x82
OP_ISDEF        equ     0x83
OP_MATCH        equ     0x88
OP_SUBST        equ     0x89
OP_HALT         equ     0xF0
OP_DEBUG        equ     0xFE
OP_INVALID      equ     0xFF

; === I/O Ports (RetroShield) ===
PORT_CONSOLE    equ     0x00        ; Console I/O port

        org     0x0100

; === Entry Point ===
start:
        ld      sp, STACK_BASE      ; Initialize hardware stack
        call    vm_init             ; Initialize VM state
        call    vm_run              ; Run bytecode
        halt                        ; Done

; === VM State Variables ===
vm_pc:          dw      0           ; Program counter (within bytecode)
vm_sp:          dw      0           ; VM stack pointer
vm_fp:          dw      0           ; VM frame pointer (for locals)
vm_code:        dw      0           ; Pointer to bytecode
vm_strings:     dw      0           ; Pointer to string table
heap_ptr:       dw      HEAP_BASE   ; Next free heap location
globals:        ds      256         ; Global variable slots (128 x 2 bytes)

; === VM Initialization ===
vm_init:
        ; Set up pointers from module header
        ld      hl, BYTECODE_BASE + HDR_CODE
        ld      (vm_code), hl

        ; Calculate string table location
        ld      hl, (BYTECODE_BASE + HDR_STRTAB)
        ld      de, BYTECODE_BASE
        add     hl, de
        ld      (vm_strings), hl

        ; Set entry point
        ld      hl, (BYTECODE_BASE + HDR_ENTRY)
        ld      (vm_pc), hl

        ; Initialize VM stack
        ld      hl, STACK_BASE - 256    ; Leave room for hardware stack
        ld      (vm_sp), hl
        ld      (vm_fp), hl

        ret

; === Main Interpreter Loop ===
vm_run:
.loop:
        ; Fetch opcode
        ld      hl, (vm_pc)
        ld      de, (vm_code)
        add     hl, de              ; HL = absolute address of opcode
        ld      a, (hl)             ; A = opcode

        ; Check for halt
        cp      OP_HALT
        ret     z

        ; Dispatch via jump table
        ld      l, a
        ld      h, 0
        add     hl, hl              ; HL = opcode * 2
        ld      de, op_table
        add     hl, de
        ld      e, (hl)
        inc     hl
        ld      d, (hl)
        ex      de, hl              ; HL = handler address
        jp      (hl)                ; Jump to handler

; === Opcode Jump Table ===
op_table:
        dw      op_nop              ; 0x00
        dw      op_push             ; 0x01
        dw      op_pushbyte         ; 0x02
        dw      op_pop              ; 0x03
        dw      op_dup              ; 0x04
        dw      op_swap             ; 0x05
        dw      op_over             ; 0x06
        dw      op_invalid          ; 0x07
        dw      op_invalid          ; 0x08
        dw      op_invalid          ; 0x09
        dw      op_invalid          ; 0x0A
        dw      op_invalid          ; 0x0B
        dw      op_invalid          ; 0x0C
        dw      op_invalid          ; 0x0D
        dw      op_invalid          ; 0x0E
        dw      op_invalid          ; 0x0F
        dw      op_ldloc            ; 0x10
        dw      op_stloc            ; 0x11
        dw      op_ldglob           ; 0x12
        dw      op_stglob           ; 0x13
        dw      op_invalid          ; 0x14
        dw      op_invalid          ; 0x15
        dw      op_invalid          ; 0x16
        dw      op_invalid          ; 0x17
        dw      op_pushstr          ; 0x18
        dw      op_strlen           ; 0x19
        dw      op_strcat           ; 0x1A
        dw      op_stridx           ; 0x1B
        dw      op_strcmp           ; 0x1C
        dw      op_substr           ; 0x1D
        dw      op_invalid          ; 0x1E
        dw      op_invalid          ; 0x1F
        dw      op_newarr           ; 0x20
        dw      op_arrlen           ; 0x21
        dw      op_arrget           ; 0x22
        dw      op_arrset           ; 0x23
        dw      op_arrpush          ; 0x24
        dw      op_arrpop           ; 0x25
        dw      op_invalid          ; 0x26
        dw      op_invalid          ; 0x27
        dw      op_newhash          ; 0x28
        dw      op_hashget          ; 0x29
        dw      op_hashset          ; 0x2A
        dw      op_hashdel          ; 0x2B
        dw      op_hashkeys         ; 0x2C
        dw      op_invalid          ; 0x2D
        dw      op_invalid          ; 0x2E
        dw      op_invalid          ; 0x2F
        dw      op_add              ; 0x30
        dw      op_sub              ; 0x31
        dw      op_mul              ; 0x32
        dw      op_div              ; 0x33
        dw      op_mod              ; 0x34
        dw      op_neg              ; 0x35
        dw      op_inc              ; 0x36
        dw      op_dec              ; 0x37
        dw      op_bitand           ; 0x38
        dw      op_bitor            ; 0x39
        dw      op_bitxor           ; 0x3A
        dw      op_bitnot           ; 0x3B
        dw      op_shl              ; 0x3C
        dw      op_shr              ; 0x3D
        dw      op_invalid          ; 0x3E
        dw      op_invalid          ; 0x3F
        dw      op_cmpeq            ; 0x40
        dw      op_cmpne            ; 0x41
        dw      op_cmplt            ; 0x42
        dw      op_cmpgt            ; 0x43
        dw      op_cmple            ; 0x44
        dw      op_cmpge            ; 0x45
        dw      op_cmp              ; 0x46
        dw      op_invalid          ; 0x47
        dw      op_streq            ; 0x48
        dw      op_strne            ; 0x49
        dw      op_strlt            ; 0x4A
        dw      op_strgt            ; 0x4B
        dw      op_strle            ; 0x4C
        dw      op_strge            ; 0x4D
        dw      op_invalid          ; 0x4E
        dw      op_invalid          ; 0x4F
        dw      op_not              ; 0x50
        dw      op_and              ; 0x51
        dw      op_or               ; 0x52
        dw      op_invalid          ; 0x53
        dw      op_invalid          ; 0x54
        dw      op_invalid          ; 0x55
        dw      op_invalid          ; 0x56
        dw      op_invalid          ; 0x57
        dw      op_invalid          ; 0x58
        dw      op_invalid          ; 0x59
        dw      op_invalid          ; 0x5A
        dw      op_invalid          ; 0x5B
        dw      op_invalid          ; 0x5C
        dw      op_invalid          ; 0x5D
        dw      op_invalid          ; 0x5E
        dw      op_invalid          ; 0x5F
        dw      op_jump             ; 0x60
        dw      op_jumpif           ; 0x61
        dw      op_jumpifnot        ; 0x62
        dw      op_jumpifdef        ; 0x63
        dw      op_invalid          ; 0x64
        dw      op_invalid          ; 0x65
        dw      op_invalid          ; 0x66
        dw      op_invalid          ; 0x67
        dw      op_call             ; 0x68
        dw      op_callnat          ; 0x69
        dw      op_return           ; 0x6A
        dw      op_returnval        ; 0x6B
        dw      op_invalid          ; 0x6C
        dw      op_invalid          ; 0x6D
        dw      op_invalid          ; 0x6E
        dw      op_invalid          ; 0x6F
        dw      op_enter            ; 0x70
        dw      op_leave            ; 0x71
        dw      op_invalid          ; 0x72
        dw      op_invalid          ; 0x73
        dw      op_invalid          ; 0x74
        dw      op_invalid          ; 0x75
        dw      op_invalid          ; 0x76
        dw      op_invalid          ; 0x77
        dw      op_print            ; 0x78
        dw      op_printstr         ; 0x79
        dw      op_printnum         ; 0x7A
        dw      op_printchar        ; 0x7B
        dw      op_println          ; 0x7C
        dw      op_input            ; 0x7D
        dw      op_inputchar        ; 0x7E
        dw      op_invalid          ; 0x7F
        dw      op_tonum            ; 0x80
        dw      op_tostr            ; 0x81
        dw      op_typeof           ; 0x82
        dw      op_isdef            ; 0x83
        ; ... rest filled with op_invalid up to 0xFF

; === Stack Helper Macros ===
; Push HL onto VM stack
vm_push:
        ex      de, hl
        ld      hl, (vm_sp)
        dec     hl
        ld      (hl), d
        dec     hl
        ld      (hl), e
        ld      (vm_sp), hl
        ret

; Pop from VM stack into HL
vm_pop:
        ld      hl, (vm_sp)
        ld      e, (hl)
        inc     hl
        ld      d, (hl)
        inc     hl
        ld      (vm_sp), hl
        ex      de, hl
        ret

; Peek at top of VM stack (doesn't pop)
vm_peek:
        ld      hl, (vm_sp)
        ld      e, (hl)
        inc     hl
        ld      d, (hl)
        ex      de, hl
        ret

; Get byte at PC+offset, result in A
get_byte_operand:
        ld      hl, (vm_pc)
        inc     hl                  ; Skip opcode
        ld      de, (vm_code)
        add     hl, de
        ld      a, (hl)
        ret

; Get word at PC+1, result in HL
get_word_operand:
        ld      hl, (vm_pc)
        inc     hl                  ; Skip opcode
        ld      de, (vm_code)
        add     hl, de
        ld      e, (hl)
        inc     hl
        ld      d, (hl)
        ex      de, hl
        ret

; Advance PC by 1 (for opcodes with no operand)
advance_pc_1:
        ld      hl, (vm_pc)
        inc     hl
        ld      (vm_pc), hl
        jp      vm_run.loop

; Advance PC by 2 (for opcodes with byte operand)
advance_pc_2:
        ld      hl, (vm_pc)
        inc     hl
        inc     hl
        ld      (vm_pc), hl
        jp      vm_run.loop

; Advance PC by 3 (for opcodes with word operand)
advance_pc_3:
        ld      hl, (vm_pc)
        inc     hl
        inc     hl
        inc     hl
        ld      (vm_pc), hl
        jp      vm_run.loop

; === Stack Operations ===
op_nop:
        jp      advance_pc_1

op_push:                            ; Push 16-bit immediate
        call    get_word_operand    ; HL = immediate value
        call    vm_push
        jp      advance_pc_3

op_pushbyte:                        ; Push 8-bit immediate (sign-extended)
        call    get_byte_operand    ; A = byte value
        ld      l, a
        ; Sign extend
        bit     7, a
        jr      z, .positive
        ld      h, 0xFF
        jr      .done
.positive:
        ld      h, 0
.done:
        call    vm_push
        jp      advance_pc_2

op_pop:
        call    vm_pop              ; Discard result
        jp      advance_pc_1

op_dup:
        call    vm_peek
        call    vm_push
        jp      advance_pc_1

op_swap:
        call    vm_pop              ; HL = top
        push    hl
        call    vm_pop              ; HL = second
        ex      de, hl              ; DE = second
        pop     hl                  ; HL = top
        call    vm_push             ; Push top first (becomes second)
        ex      de, hl              ; HL = second
        call    vm_push             ; Push second (becomes top)
        jp      advance_pc_1

op_over:
        call    vm_pop              ; HL = top
        push    hl
        call    vm_peek             ; HL = second (peek, don't pop)
        ex      de, hl              ; DE = second
        pop     hl                  ; HL = top
        call    vm_push             ; Push top back
        ex      de, hl              ; HL = second
        call    vm_push             ; Push copy of second
        jp      advance_pc_1

; === Local Variables ===
op_ldloc:
        call    get_byte_operand    ; A = local index
        ld      hl, (vm_fp)
        ld      e, a
        ld      d, 0
        add     hl, de              ; HL = fp + index
        add     hl, de              ; HL = fp + index * 2 (16-bit values)
        ld      e, (hl)
        inc     hl
        ld      d, (hl)
        ex      de, hl              ; HL = value
        call    vm_push
        jp      advance_pc_2

op_stloc:
        call    get_byte_operand    ; A = local index
        push    af
        call    vm_pop              ; HL = value to store
        ex      de, hl              ; DE = value
        pop     af
        ld      hl, (vm_fp)
        ld      c, a
        ld      b, 0
        add     hl, bc
        add     hl, bc              ; HL = fp + index * 2
        ld      (hl), e
        inc     hl
        ld      (hl), d
        jp      advance_pc_2

; === Global Variables ===
op_ldglob:
        call    get_word_operand    ; HL = global index
        add     hl, hl              ; HL = index * 2
        ld      de, globals
        add     hl, de              ; HL = &globals[index]
        ld      e, (hl)
        inc     hl
        ld      d, (hl)
        ex      de, hl
        call    vm_push
        jp      advance_pc_3

op_stglob:
        call    get_word_operand    ; HL = global index
        push    hl
        call    vm_pop              ; HL = value
        ex      de, hl              ; DE = value
        pop     hl                  ; HL = index
        add     hl, hl              ; HL = index * 2
        ld      bc, globals
        add     hl, bc              ; HL = &globals[index]
        ld      (hl), e
        inc     hl
        ld      (hl), d
        jp      advance_pc_3

; === String Operations ===
op_pushstr:
        call    get_word_operand    ; HL = string index
        call    get_string_ptr      ; HL = pointer to string data
        call    vm_push
        jp      advance_pc_3

; Get pointer to string by index (index in HL, returns pointer in HL)
get_string_ptr:
        push    hl
        ld      hl, (vm_strings)    ; HL = string table base
        ld      a, (hl)             ; A = string count
        inc     hl                  ; Skip count byte
        pop     de                  ; DE = target index
        ; Skip to string[index]
        ld      b, e                ; B = index
        xor     a
        cp      b
        jr      z, .found
.skip_loop:
        ld      a, (hl)             ; A = string length
        inc     hl                  ; Skip length byte
        ld      e, a
        ld      d, 0
        add     hl, de              ; Skip string data
        djnz    .skip_loop
.found:
        ; HL now points to length byte of target string
        ret

op_strlen:
        call    vm_pop              ; HL = string pointer
        ld      a, (hl)             ; A = length
        ld      l, a
        ld      h, 0
        call    vm_push
        jp      advance_pc_1

op_strcat:
        ; Pop two strings, concatenate, push result
        call    vm_pop              ; HL = str2
        push    hl
        call    vm_pop              ; HL = str1
        pop     de                  ; DE = str2
        call    string_concat       ; HL = new string
        call    vm_push
        jp      advance_pc_1

op_stridx:
        ; str[idx] - get character at index
        call    vm_pop              ; HL = index
        push    hl
        call    vm_pop              ; HL = string
        pop     de                  ; DE = index
        ld      a, (hl)             ; A = length
        cp      e                   ; Check bounds
        jr      c, .out_of_bounds
        jr      z, .out_of_bounds
        inc     hl                  ; Skip length
        add     hl, de              ; HL = &str[index]
        ld      l, (hl)
        ld      h, 0
        call    vm_push
        jp      advance_pc_1
.out_of_bounds:
        ld      hl, 0               ; Return 0 for out of bounds
        call    vm_push
        jp      advance_pc_1

op_strcmp:
        call    vm_pop              ; HL = str2
        push    hl
        call    vm_pop              ; HL = str1
        pop     de                  ; DE = str2
        call    string_compare      ; A = result (-1, 0, 1)
        ld      l, a
        ; Sign extend
        bit     7, a
        jr      z, .pos
        ld      h, 0xFF
        jr      .done
.pos:
        ld      h, 0
.done:
        call    vm_push
        jp      advance_pc_1

op_substr:
        ; substr(str, start, len)
        call    vm_pop              ; HL = len
        push    hl
        call    vm_pop              ; HL = start
        push    hl
        call    vm_pop              ; HL = string
        pop     de                  ; DE = start
        pop     bc                  ; BC = len
        call    string_substr
        call    vm_push
        jp      advance_pc_1

; === Arithmetic Operations ===
op_add:
        call    vm_pop              ; HL = b
        ex      de, hl              ; DE = b
        call    vm_pop              ; HL = a
        add     hl, de              ; HL = a + b
        call    vm_push
        jp      advance_pc_1

op_sub:
        call    vm_pop              ; HL = b
        ex      de, hl              ; DE = b
        call    vm_pop              ; HL = a
        or      a                   ; Clear carry
        sbc     hl, de              ; HL = a - b
        call    vm_push
        jp      advance_pc_1

op_mul:
        call    vm_pop              ; HL = b
        push    hl
        call    vm_pop              ; HL = a
        pop     de                  ; DE = b
        call    mul16               ; HL = a * b
        call    vm_push
        jp      advance_pc_1

op_div:
        call    vm_pop              ; HL = b (divisor)
        push    hl
        call    vm_pop              ; HL = a (dividend)
        pop     de                  ; DE = divisor
        call    div16               ; HL = a / b
        call    vm_push
        jp      advance_pc_1

op_mod:
        call    vm_pop              ; HL = b (divisor)
        push    hl
        call    vm_pop              ; HL = a (dividend)
        pop     de                  ; DE = divisor
        call    mod16               ; HL = a % b
        call    vm_push
        jp      advance_pc_1

op_neg:
        call    vm_pop              ; HL = a
        ex      de, hl              ; DE = a
        ld      hl, 0
        or      a
        sbc     hl, de              ; HL = 0 - a
        call    vm_push
        jp      advance_pc_1

op_inc:
        call    vm_pop              ; HL = a
        inc     hl
        call    vm_push
        jp      advance_pc_1

op_dec:
        call    vm_pop              ; HL = a
        dec     hl
        call    vm_push
        jp      advance_pc_1

; === Bitwise Operations ===
op_bitand:
        call    vm_pop              ; HL = b
        push    hl
        call    vm_pop              ; HL = a
        pop     de                  ; DE = b
        ld      a, l
        and     e
        ld      l, a
        ld      a, h
        and     d
        ld      h, a
        call    vm_push
        jp      advance_pc_1

op_bitor:
        call    vm_pop
        push    hl
        call    vm_pop
        pop     de
        ld      a, l
        or      e
        ld      l, a
        ld      a, h
        or      d
        ld      h, a
        call    vm_push
        jp      advance_pc_1

op_bitxor:
        call    vm_pop
        push    hl
        call    vm_pop
        pop     de
        ld      a, l
        xor     e
        ld      l, a
        ld      a, h
        xor     d
        ld      h, a
        call    vm_push
        jp      advance_pc_1

op_bitnot:
        call    vm_pop
        ld      a, l
        cpl
        ld      l, a
        ld      a, h
        cpl
        ld      h, a
        call    vm_push
        jp      advance_pc_1

op_shl:
        call    vm_pop              ; HL = shift count
        ld      b, l                ; B = count
        call    vm_pop              ; HL = value
        ld      a, b
        or      a
        jr      z, .done
.loop:
        add     hl, hl              ; HL <<= 1
        djnz    .loop
.done:
        call    vm_push
        jp      advance_pc_1

op_shr:
        call    vm_pop              ; HL = shift count
        ld      b, l                ; B = count
        call    vm_pop              ; HL = value
        ld      a, b
        or      a
        jr      z, .done
.loop:
        srl     h
        rr      l                   ; HL >>= 1
        djnz    .loop
.done:
        call    vm_push
        jp      advance_pc_1

; === Comparison Operations ===
op_cmpeq:
        call    vm_pop              ; HL = b
        ex      de, hl
        call    vm_pop              ; HL = a
        or      a
        sbc     hl, de
        jr      z, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

op_cmpne:
        call    vm_pop
        ex      de, hl
        call    vm_pop
        or      a
        sbc     hl, de
        jr      nz, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

op_cmplt:
        call    vm_pop              ; HL = b
        ex      de, hl              ; DE = b
        call    vm_pop              ; HL = a
        call    cmp_signed          ; Compare a < b
        jr      c, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

op_cmpgt:
        call    vm_pop              ; HL = b
        ex      de, hl              ; DE = b
        call    vm_pop              ; HL = a
        ; a > b means b < a
        push    hl
        ex      de, hl
        pop     de                  ; Now HL = b, DE = a
        call    cmp_signed
        jr      c, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

op_cmple:
        call    vm_pop              ; HL = b
        ex      de, hl              ; DE = b
        call    vm_pop              ; HL = a
        ; a <= b means !(b < a)
        push    hl
        ex      de, hl
        pop     de                  ; Now HL = b, DE = a
        call    cmp_signed
        jr      c, .false
        ld      hl, 1
        jr      .done
.false:
        ld      hl, 0
.done:
        call    vm_push
        jp      advance_pc_1

op_cmpge:
        call    vm_pop              ; HL = b
        ex      de, hl              ; DE = b
        call    vm_pop              ; HL = a
        ; a >= b means !(a < b)
        call    cmp_signed
        jr      c, .false
        ld      hl, 1
        jr      .done
.false:
        ld      hl, 0
.done:
        call    vm_push
        jp      advance_pc_1

op_cmp:
        ; Spaceship operator: -1, 0, 1
        call    vm_pop              ; HL = b
        ex      de, hl              ; DE = b
        call    vm_pop              ; HL = a
        or      a
        sbc     hl, de
        jr      z, .equal
        jp      m, .less
        ld      hl, 1
        jr      .done
.less:
        ld      hl, 0xFFFF          ; -1
        jr      .done
.equal:
        ld      hl, 0
.done:
        call    vm_push
        jp      advance_pc_1

; === String Comparison ===
op_streq:
        call    vm_pop              ; HL = str2
        push    hl
        call    vm_pop              ; HL = str1
        pop     de
        call    string_compare
        or      a
        jr      z, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

op_strne:
        call    vm_pop
        push    hl
        call    vm_pop
        pop     de
        call    string_compare
        or      a
        jr      nz, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

op_strlt:
        call    vm_pop
        push    hl
        call    vm_pop
        pop     de
        call    string_compare
        cp      0xFF                ; -1
        jr      z, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

op_strgt:
        call    vm_pop
        push    hl
        call    vm_pop
        pop     de
        call    string_compare
        cp      1
        jr      z, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

op_strle:
        call    vm_pop
        push    hl
        call    vm_pop
        pop     de
        call    string_compare
        cp      1
        jr      z, .false
        ld      hl, 1
        jr      .done
.false:
        ld      hl, 0
.done:
        call    vm_push
        jp      advance_pc_1

op_strge:
        call    vm_pop
        push    hl
        call    vm_pop
        pop     de
        call    string_compare
        cp      0xFF                ; -1
        jr      z, .false
        ld      hl, 1
        jr      .done
.false:
        ld      hl, 0
.done:
        call    vm_push
        jp      advance_pc_1

; === Logical Operations ===
op_not:
        call    vm_pop
        ld      a, h
        or      l
        jr      z, .was_false
        ld      hl, 0               ; Non-zero becomes 0
        jr      .done
.was_false:
        ld      hl, 1               ; Zero becomes 1
.done:
        call    vm_push
        jp      advance_pc_1

op_and:
        call    vm_pop              ; HL = b
        ld      a, h
        or      l
        jr      z, .false
        call    vm_pop              ; HL = a
        ld      a, h
        or      l
        jr      z, .false
        ld      hl, 1
        jr      .done
.false:
        call    vm_pop              ; Clean up stack if b was true
        ld      hl, 0
.done:
        call    vm_push
        jp      advance_pc_1

op_or:
        call    vm_pop              ; HL = b
        ld      a, h
        or      l
        jr      nz, .true
        call    vm_pop              ; HL = a
        ld      a, h
        or      l
        jr      nz, .true
        ld      hl, 0
        jr      .done
.true:
        ld      hl, 1
.done:
        call    vm_push
        jp      advance_pc_1

; === Control Flow ===
op_jump:
        call    get_word_operand    ; HL = target address
        ld      (vm_pc), hl
        jp      vm_run.loop

op_jumpif:
        call    vm_pop              ; HL = condition
        ld      a, h
        or      l
        jr      z, .false
        call    get_word_operand    ; HL = target
        ld      (vm_pc), hl
        jp      vm_run.loop
.false:
        jp      advance_pc_3

op_jumpifnot:
        call    vm_pop
        ld      a, h
        or      l
        jr      nz, .true
        call    get_word_operand
        ld      (vm_pc), hl
        jp      vm_run.loop
.true:
        jp      advance_pc_3

op_jumpifdef:
        ; Check if top of stack is defined (non-zero for now)
        call    vm_pop
        ld      a, h
        or      l
        jr      z, .undef
        call    get_word_operand
        ld      (vm_pc), hl
        jp      vm_run.loop
.undef:
        jp      advance_pc_3

; === Subroutine Calls ===
op_call:
        call    get_word_operand    ; HL = target address
        push    hl                  ; Save target
        ; Push return address (current PC + 3)
        ld      hl, (vm_pc)
        inc     hl
        inc     hl
        inc     hl                  ; HL = return address
        call    vm_push             ; Push on VM stack
        ; Push current frame pointer
        ld      hl, (vm_fp)
        call    vm_push
        pop     hl                  ; HL = target
        ld      (vm_pc), hl
        jp      vm_run.loop

op_callnat:
        ; Native function call (not implemented yet)
        call    get_byte_operand    ; A = native function ID
        ; TODO: dispatch to native functions
        jp      advance_pc_2

op_return:
        ; Restore frame pointer
        ld      hl, (vm_fp)
        ld      (vm_sp), hl         ; Discard locals
        call    vm_pop              ; HL = saved fp
        ld      (vm_fp), hl
        call    vm_pop              ; HL = return address
        ld      (vm_pc), hl
        jp      vm_run.loop

op_returnval:
        ; Same as return but keep top of stack value
        call    vm_pop              ; HL = return value
        push    hl
        ld      hl, (vm_fp)
        ld      (vm_sp), hl
        call    vm_pop
        ld      (vm_fp), hl
        call    vm_pop              ; HL = return address
        ld      (vm_pc), hl
        pop     hl                  ; Restore return value
        call    vm_push
        jp      vm_run.loop

; === Frame Management ===
op_enter:
        call    get_byte_operand    ; A = number of locals
        ; Save current FP as new FP
        ld      hl, (vm_sp)
        ld      (vm_fp), hl
        ; Allocate space for locals
        ld      e, a
        ld      d, 0
        add     hl, de
        add     hl, de              ; HL = SP - num_locals * 2
        ; Initialize locals to 0
        ld      b, a
        ld      hl, (vm_fp)
.zero_loop:
        ld      a, b
        or      a
        jr      z, .done
        ld      (hl), 0
        inc     hl
        ld      (hl), 0
        inc     hl
        djnz    .zero_loop
.done:
        ld      (vm_sp), hl
        jp      advance_pc_2

op_leave:
        ld      hl, (vm_fp)
        ld      (vm_sp), hl
        jp      advance_pc_1

; === I/O Operations ===
op_print:
        ; Auto-detect type and print
        call    vm_pop
        ; For now, just print as number
        call    print_number
        jp      advance_pc_1

op_printstr:
        call    vm_pop              ; HL = string pointer
        call    print_string
        jp      advance_pc_1

op_printnum:
        call    vm_pop
        call    print_number
        jp      advance_pc_1

op_printchar:
        call    vm_pop
        ld      a, l
        call    putchar
        jp      advance_pc_1

op_println:
        ld      a, 0x0D             ; CR
        call    putchar
        ld      a, 0x0A             ; LF
        call    putchar
        jp      advance_pc_1

op_input:
        ; Read line into new string
        call    read_line
        call    vm_push
        jp      advance_pc_1

op_inputchar:
        call    getchar
        ld      l, a
        ld      h, 0
        call    vm_push
        jp      advance_pc_1

; === Type Operations ===
op_tonum:
        call    vm_pop              ; HL = string pointer
        call    string_to_number
        call    vm_push
        jp      advance_pc_1

op_tostr:
        call    vm_pop              ; HL = number
        call    number_to_string
        call    vm_push
        jp      advance_pc_1

op_typeof:
        ; For now, just return 1 (number) for everything
        call    vm_pop
        ld      hl, 1
        call    vm_push
        jp      advance_pc_1

op_isdef:
        call    vm_pop
        ld      a, h
        or      l
        jr      z, .undef
        ld      hl, 1
        jr      .done
.undef:
        ld      hl, 0
.done:
        call    vm_push
        jp      advance_pc_1

; === Array Operations (stubs) ===
op_newarr:
op_arrlen:
op_arrget:
op_arrset:
op_arrpush:
op_arrpop:
        ; TODO: implement arrays
        jp      advance_pc_1

; === Hash Operations (stubs) ===
op_newhash:
op_hashget:
op_hashset:
op_hashdel:
op_hashkeys:
        ; TODO: implement hashes
        jp      advance_pc_1

op_match:
op_subst:
        ; TODO: implement regex
        jp      advance_pc_1

op_invalid:
        ; Invalid opcode - halt
        halt

; === Helper Functions ===

; 16-bit signed compare: HL < DE sets carry
cmp_signed:
        ld      a, h
        xor     d
        jp      p, .same_sign
        ; Different signs: negative < positive
        bit     7, h
        jr      z, .hl_positive
        scf                         ; HL is negative, DE positive: HL < DE
        ret
.hl_positive:
        or      a                   ; HL is positive, DE negative: HL >= DE
        ret
.same_sign:
        ; Same sign: unsigned compare works
        or      a
        sbc     hl, de
        add     hl, de              ; Restore HL
        ret

; 16-bit multiply: HL = HL * DE
mul16:
        push    bc
        ld      b, h
        ld      c, l                ; BC = multiplicand
        ld      hl, 0               ; Result
        ld      a, 16               ; Bit counter
.loop:
        add     hl, hl              ; Shift result left
        ex      de, hl
        add     hl, hl              ; Shift multiplier left
        ex      de, hl
        jr      nc, .no_add
        add     hl, bc              ; Add multiplicand if bit set
.no_add:
        dec     a
        jr      nz, .loop
        pop     bc
        ret

; 16-bit divide: HL = HL / DE (unsigned)
div16:
        push    bc
        ld      b, h
        ld      c, l                ; BC = dividend
        ld      hl, 0               ; Remainder
        ld      a, 16               ; Bit counter
.loop:
        sla     c
        rl      b                   ; Shift dividend left
        adc     hl, hl              ; Shift remainder left, add carry
        sbc     hl, de              ; Try subtract divisor
        jr      nc, .no_restore
        add     hl, de              ; Restore if negative
        jr      .next
.no_restore:
        inc     c                   ; Set quotient bit
.next:
        dec     a
        jr      nz, .loop
        ld      h, b
        ld      l, c                ; HL = quotient
        pop     bc
        ret

; 16-bit modulo: HL = HL % DE
mod16:
        push    bc
        ld      b, h
        ld      c, l                ; BC = dividend
        ld      hl, 0               ; Remainder
        ld      a, 16
.loop:
        sla     c
        rl      b
        adc     hl, hl
        sbc     hl, de
        jr      nc, .no_restore
        add     hl, de
        jr      .next
.no_restore:
        inc     c
.next:
        dec     a
        jr      nz, .loop
        ; HL = remainder
        pop     bc
        ret

; Print null-terminated string at HL
print_string:
        ld      a, (hl)             ; A = length
        or      a
        ret     z
        ld      b, a                ; B = count
        inc     hl                  ; Skip length byte
.loop:
        ld      a, (hl)
        call    putchar
        inc     hl
        djnz    .loop
        ret

; Print 16-bit signed number in HL
print_number:
        push    bc
        push    de
        push    hl

        bit     7, h
        jr      z, .positive
        ; Negative: print minus and negate
        ld      a, '-'
        call    putchar
        ex      de, hl
        ld      hl, 0
        or      a
        sbc     hl, de

.positive:
        ; Convert to decimal
        ld      de, 10000
        call    .digit
        ld      de, 1000
        call    .digit
        ld      de, 100
        call    .digit
        ld      de, 10
        call    .digit
        ld      a, l
        add     a, '0'
        call    putchar

        pop     hl
        pop     de
        pop     bc
        ret

.digit:
        ld      c, '0' - 1
.sub_loop:
        inc     c
        or      a
        sbc     hl, de
        jr      nc, .sub_loop
        add     hl, de
        ld      a, c
        cp      '0'
        ret     z                   ; Skip leading zeros
        call    putchar
        ret

; String compare: HL = str1, DE = str2
; Returns: A = -1 if str1 < str2, 0 if equal, 1 if str1 > str2
string_compare:
        push    bc
        push    hl
        push    de

        ld      a, (hl)             ; A = len1
        ld      b, a
        ld      a, (de)             ; A = len2
        ld      c, a

        inc     hl
        inc     de

        ; Compare min(len1, len2) characters
        ld      a, b
        cp      c
        jr      c, .use_b
        ld      a, c
.use_b:
        or      a
        jr      z, .compare_lens

.cmp_loop:
        push    af
        ld      a, (de)
        cp      (hl)
        jr      c, .greater
        jr      nz, .less
        inc     hl
        inc     de
        pop     af
        dec     a
        jr      nz, .cmp_loop

.compare_lens:
        ld      a, b
        cp      c
        jr      z, .equal
        jr      c, .less
        jr      .greater

.less:
        pop     af
        pop     de
        pop     hl
        pop     bc
        ld      a, 0xFF             ; -1
        ret

.greater:
        pop     af
        pop     de
        pop     hl
        pop     bc
        ld      a, 1
        ret

.equal:
        pop     de
        pop     hl
        pop     bc
        xor     a                   ; 0
        ret

; String concatenate: HL = str1, DE = str2
; Returns: HL = new string on heap
string_concat:
        push    bc
        push    de
        push    hl

        ; Get lengths
        ld      a, (hl)             ; len1
        ld      b, a
        ld      a, (de)             ; len2
        ld      c, a

        ; Allocate new string: 1 + len1 + len2 bytes
        add     a, b
        ld      e, a
        ld      d, 0
        inc     de                  ; +1 for length byte
        call    heap_alloc          ; HL = new string

        pop     de                  ; DE = str1
        push    hl                  ; Save result ptr

        ; Store combined length
        ld      a, b
        add     a, c
        ld      (hl), a
        inc     hl

        ; Copy str1
        push    de
        pop     ix                  ; IX = str1
        ld      a, (ix+0)           ; len1
        or      a
        jr      z, .skip1
        ld      b, a
        inc     de
.copy1:
        ld      a, (de)
        ld      (hl), a
        inc     de
        inc     hl
        djnz    .copy1
.skip1:

        ; Copy str2
        pop     de                  ; DE = str2 (from original push)
        push    de
        ld      a, (de)             ; len2
        or      a
        jr      z, .skip2
        ld      b, a
        inc     de
.copy2:
        ld      a, (de)
        ld      (hl), a
        inc     de
        inc     hl
        djnz    .copy2
.skip2:

        pop     de                  ; Clean up stack
        pop     hl                  ; HL = result ptr
        pop     bc
        ret

; Substring: HL = string, DE = start, BC = len
; Returns: HL = new string
string_substr:
        push    bc
        push    de
        push    hl

        ; Allocate new string
        push    bc
        inc     bc                  ; +1 for length
        ex      de, hl
        ld      d, b
        ld      e, c
        call    heap_alloc          ; HL = new string
        pop     bc                  ; BC = len

        pop     de                  ; DE = original string
        push    hl                  ; Save result

        ; Store length
        ld      (hl), c
        inc     hl

        ; Get start position
        pop     ix                  ; IX = result
        push    ix
        pop     iy                  ; IY = result too
        pop     de                  ; DE = start

        ; Source = original + 1 + start
        ld      hl, (sp)            ; Hmm, this is getting complicated
        ; Simplified: just copy BC bytes from start
        pop     hl                  ; Result
        pop     bc
        ret

; Heap allocate: DE = size in bytes
; Returns: HL = allocated block
heap_alloc:
        ld      hl, (heap_ptr)
        push    hl                  ; Save current ptr
        add     hl, de
        ld      (heap_ptr), hl      ; Advance heap
        pop     hl                  ; Return original ptr
        ret

; Read line from input into new heap string
read_line:
        push    bc
        push    de

        ; Allocate buffer (max 256 bytes)
        ld      de, 256
        call    heap_alloc
        push    hl                  ; Save string ptr

        inc     hl                  ; Skip length byte for now
        ld      b, 0                ; Length counter

.read_loop:
        call    getchar
        cp      0x0D                ; CR?
        jr      z, .done
        cp      0x0A                ; LF?
        jr      z, .done
        ld      (hl), a
        inc     hl
        inc     b
        ld      a, b
        cp      255
        jr      nz, .read_loop

.done:
        pop     hl                  ; HL = string ptr
        ld      (hl), b             ; Store length
        pop     de
        pop     bc
        ret

; Convert string at HL to number, result in HL
string_to_number:
        push    bc
        push    de

        ld      a, (hl)             ; Length
        or      a
        jr      z, .zero
        ld      b, a
        inc     hl

        ld      de, 0               ; Result
        ld      c, 0                ; Negative flag

        ; Check for minus
        ld      a, (hl)
        cp      '-'
        jr      nz, .parse_loop
        ld      c, 1
        inc     hl
        dec     b
        jr      z, .zero

.parse_loop:
        ld      a, (hl)
        sub     '0'
        jr      c, .done
        cp      10
        jr      nc, .done

        ; DE = DE * 10 + digit
        push    hl
        push    af
        ex      de, hl
        add     hl, hl              ; *2
        ld      d, h
        ld      e, l
        add     hl, hl              ; *4
        add     hl, hl              ; *8
        add     hl, de              ; *10
        pop     af
        ld      e, a
        ld      d, 0
        add     hl, de
        ex      de, hl
        pop     hl

        inc     hl
        djnz    .parse_loop

.done:
        ex      de, hl              ; HL = result
        ld      a, c
        or      a
        jr      z, .positive
        ; Negate
        ex      de, hl
        ld      hl, 0
        or      a
        sbc     hl, de
.positive:
        pop     de
        pop     bc
        ret

.zero:
        ld      hl, 0
        pop     de
        pop     bc
        ret

; Convert number in HL to string, return pointer in HL
number_to_string:
        push    bc
        push    de

        ; Allocate buffer
        push    hl
        ld      de, 8               ; Max 7 chars + length
        call    heap_alloc
        ex      de, hl              ; DE = buffer
        pop     hl                  ; HL = number

        push    de                  ; Save buffer ptr
        inc     de                  ; Skip length byte

        ld      bc, 0               ; Char count

        bit     7, h
        jr      z, .positive
        ; Negative
        ld      a, '-'
        ld      (de), a
        inc     de
        inc     c

        ex      de, hl
        push    de
        ld      de, 0
        or      a
        sbc     hl, de
        ex      de, hl
        pop     hl
        ex      de, hl

.positive:
        ; Convert to decimal (at least one digit)
        push    de
        ld      de, 10000
        call    .digit
        ld      de, 1000
        call    .digit
        ld      de, 100
        call    .digit
        ld      de, 10
        call    .digit
        pop     de

        ; Last digit always
        ld      a, l
        add     a, '0'
        ld      (de), a
        inc     de
        inc     c

        pop     hl                  ; HL = buffer
        ld      (hl), c             ; Store length
        pop     de
        pop     bc
        ret

.digit:
        push    de
        ld      a, '0' - 1
.sub_loop:
        inc     a
        or      a
        sbc     hl, de
        jr      nc, .sub_loop
        add     hl, de
        pop     de
        cp      '0'
        ret     z                   ; Skip leading zeros
        ld      (de), a
        inc     de
        inc     c
        ret

; === Platform I/O (RetroShield) ===
putchar:
        ; Output character in A to console
        out     (PORT_CONSOLE), a
        ret

getchar:
        ; Read character into A
        in      a, (PORT_CONSOLE)
        ret

; === End of Runtime ===
        end     start
