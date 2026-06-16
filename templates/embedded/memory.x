/* Linker memory layout. Adjust ORIGIN/LENGTH for your specific MCU. */
MEMORY
{
  /* Defaults below match an STM32F411RE (512K flash, 128K RAM). */
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
}
