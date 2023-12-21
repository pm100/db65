; Verifies that sim65 can handle 64-bit timeout counter.
; sim65 sim65-timein.prg -x 4400000000

.export _main
.import exit

_main:
    ; wait ~4,300,000,000 cycles
    lda #43

    lda #1
    rts


