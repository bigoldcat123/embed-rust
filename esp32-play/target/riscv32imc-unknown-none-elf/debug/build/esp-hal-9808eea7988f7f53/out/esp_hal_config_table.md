
| Name | Description | Default&nbsp;value | Allowed&nbsp;value |
|------|-------------|--------------------|--------------------|
| <p>ESP_HAL_CONFIG_PLACE_SPI_MASTER_DRIVER_IN_RAM</p><p>⚠️ Unstable</p> | <p>Places the SPI master driver in RAM for better performance</p> | <center>false</center> | <center></center>
| <p>ESP_HAL_CONFIG_PLACE_SWITCH_TABLES_IN_RAM</p><p>Stable since 1.0.0-beta.0</p> | <p>Places switch-tables, some lookup tables and constants related to interrupt handling into RAM - resulting in better performance but slightly more RAM consumption.</p> | <center>true</center> | <center></center>
| <p>ESP_HAL_CONFIG_PLACE_ANON_IN_RAM</p><p>Stable since 1.0.0-beta.0</p> | <p>Places anonymous symbols into RAM - resulting in better performance at the cost of significant more RAM consumption. Best to be combined with `place-switch-tables-in-ram`.</p> | <center>false</center> | <center></center>
| <p>ESP_HAL_CONFIG_STACK_GUARD_OFFSET</p><p>Stable since 1.0.0-beta.0</p> | <p>The stack guard variable will be placed this many bytes from the stack's end.</p> | <center>4096</center> | <center></center>
| <p>ESP_HAL_CONFIG_STACK_GUARD_VALUE</p><p>Stable since 1.0.0-beta.0</p> | <p>The value to be written to the stack guard variable.</p> | <center>3740121773</center> | <center></center>
| <p>ESP_HAL_CONFIG_IMPL_CRITICAL_SECTION</p><p>⚠️ Unstable</p> | <p>Provide a `critical-section` implementation. Note that if disabled, you will need to provide a `critical-section` implementation which is using `restore-state-u32`.</p> | <center>true</center> | <center></center>
