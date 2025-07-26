MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* FLASH : ORIGIN = 0x00000000, LENGTH = 1024K */ 
  /* RAM : ORIGIN = 0x20000000, LENGTH = 256K */ 

  /* These values correspond to the NRF52840 with Softdevices S140 7.3.0 */
  
  /* FLASH : ORIGIN = 0x00027000, LENGTH = 868K */
  /* RAM : ORIGIN = 0x20020000, LENGTH = 128K */

  /* NOTE 1 K = 1 KiB = 1024 bytes */
  /* These values correspond to the nRF52840 WITH Adafruit nRF52 bootloader */
  FLASH : ORIGIN = 0x00001000, LENGTH = 1020K
  RAM : ORIGIN = 0x20000008, LENGTH = 255K
  
}
