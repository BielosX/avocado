MEMORY
{
  CCMRAM    (xrw)    : ORIGIN = 0x10000000,   LENGTH = 64K
  RAM    (xrw)    : ORIGIN = 0x20000000,   LENGTH = 192K
  FLASH    (rx)    : ORIGIN = 0x8000000,   LENGTH = 2048K
}

_initial_stack_pointer_value = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(reset);
EXTERN(RESET_HANDLER);

SECTIONS
{
  .vector_table ORIGIN(FLASH) :
  {
    LONG(_initial_stack_pointer_value);
    KEEP(*(.vector_table.reset));
    FILL(0)
    . += 216;
    KEEP(*(.vector_table.exti15_10));
  } >FLASH

  .text :
  {
    *(.text .text.*)
  } >FLASH

  .rodata :
  {
    *(.rodata)
    *(.rodata.*)
  } >FLASH
}