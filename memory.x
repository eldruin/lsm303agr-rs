# Linker script for the nRF52833 MCU on the microbit:v2 necessary for the example. Please disregard it otherwise.
MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x00000000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}
