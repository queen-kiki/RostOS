ENTRY(_start)

SECTIONS
{
  . = 0xFFFF800000000000;
  .text : { *(.text .text.*) }
  . = ALIGN(0x1000);
  .rodata : { *(.rodata .rodata.*) }
  . = ALIGN(0x1000);
  .eh_frame : { *(.eh_frame) }
  . = ALIGN(0x1000);
  .data : { *(.data .data.*) }
  . = ALIGN(0x1000);
  .data.rel.ro : { *(.data.rel.ro .data.rel.ro.*)}
  . = ALIGN(0x1000);
  .got : { *(.got) *(.got.plt)}
  . = ALIGN(0x1000);
  .bss : { *(.bss .bss.*) *(COMMON) }
  /DISCARD/ : { *(.comment) *(.debug_loc) *(.debug_str) *(.debug_abbrev) *(.debug_info) *(.debug_macinfo) *(.debug_pubnames) *(.debug_pubtypes)  *(.debug_frame)  *(.debug_line)  *(.debug_ranges) }

}
