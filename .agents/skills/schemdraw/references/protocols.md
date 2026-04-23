# Protocol Harness Patterns

Use this reference when the drawing is primarily about a protocol or debug/programming interface rather than a generic connector family.

## Local Rule

Define protocol links as:

1. a fixed signal inventory
2. an endpoint family
3. a required mapping set
4. a rendered view

Do not start from hand-drawn connector art when the real job is validating a protocol contract.

## Bundled Protocol Examples

- `examples/swd_programming.py`: 2-wire debug/programming plus reset, reference voltage, and ground
- `examples/jtag_fpga.py`: 4-wire FPGA JTAG plus reference and ground
- `examples/arm20_swd_header.py`: named ARM 20-pin SWD physical header pattern
- `examples/arm20_jtag_header.py`: named ARM 20-pin JTAG physical header pattern
- `examples/cortex9_swd_header.py`: named Cortex 9-pin SWD/JTAG physical header pattern
- `examples/spi_peripheral.py`: clock, chip-enable, MOSI, MISO, power, and ground
- `examples/uart_serial.py`: simple UART serial plus power and ground
- `examples/i2c_sensor.py`: short-reach I2C bus pattern
- `examples/i2c_multidrop.py`: controller-plus-multiple-target I2C bus pattern with explicit pull-up ownership
- `examples/qwiic_i2c_link.py`: exact Qwiic JST-SH 4-pin I2C pin-map pattern
- `examples/grove_i2c_link.py`: exact Grove 4-pin I2C pin-map pattern
- `examples/onewire_sensor.py`: 1-Wire device link pattern
- `examples/mdio_link.py`: MDIO management link pattern
- `examples/rs422_link.py`: full-duplex differential serial with shield policy
- `examples/rs485_bus.py`: 2-wire differential bus segment with shield policy
- `examples/rs485_multidrop.py`: multidrop RS-485 bus with explicit end/drop roles, bias, and termination policy
- `examples/spacewire_link.py`: bidirectional data/strobe differential pairs and ground
- `examples/ethernet_link.py`: RJ45 T568B logical link
- `examples/ethernet_poe_link.py`: shielded PoE-aware Ethernet logical link
- `examples/pps_sync.py`: PPS timing link plus power and ground

## SWD

Typical logical signals for the local helper path:

- `VTREF`
- `SWDIO`
- `SWCLK`
- optional `SWO`
- optional `NRST`
- `GND`

Primary source used for the local defaults:

- ARM DSTREAM-ST SWD reference material describing `SWDIO` as bidirectional and showing `VTREF`, `SWDIO`, `SWCLK`, optional `SWO`, `nSRST`, and `GND`
- SEGGER 20-pin and 9-pin debug connector knowledge-base pinouts for the named physical header examples

## JTAG

Typical logical signals for the local helper path:

- `VTREF`
- `TMS`
- `TCK`
- `TDI`
- `TDO`
- optional `TRST_N`
- optional `SRST_N`
- `GND`

Primary source used for the local defaults:

- vendor JTAG references describing mandatory `TMS`, `TCK`, `TDI`, and `TDO`, with reset signals varying by target family
- SEGGER 20-pin JTAG connector pinout for the named ARM-standard physical header example

## Named Physical Standards

The local helper layer now includes exact pin-map patterns for:

- ARM 20-pin SWD
- ARM 20-pin JTAG
- Cortex 9-pin SWD/JTAG
- DE-9 RS-232
- RJ45 T568B
- Qwiic JST-SH 4-pin I2C
- Grove 4-pin I2C

Use these when:

- the connector itself is part of the interface contract
- pin numbering matters as much as signal naming
- you want schema validation to catch physical pin-map drift, not just logical signal drift

## SPI

Typical logical signals for the local helper path:

- `VCC`
- `CS_N`
- `SCLK`
- `MOSI`
- `MISO`
- `GND`

Use this for:

- microcontroller peripheral links
- sensor/control buses represented as one controller to one device

## UART

The local helper uses:

- `VCC`
- `TX`
- `RX`
- `GND`

Use this for:

- point-to-point serial debug or console links
- simple ICDs where logical serial naming matters more than exact board header layout

## I2C

The local helper uses:

- `VCC`
- `SCL`
- `SDA`
- `GND`

Use this for:

- short-reach controller-to-sensor links
- ICDs where the important contract is that the shared clock/data pair and reference supply are explicit

Design rule:

- the local example models the logical bus segment, not pull-up placement or multi-drop address policy

Multidrop rule:

- use `examples/i2c_multidrop.py` when the important contract is bus ownership, not just one controller-to-one-target wiring segment
- make one controller own pull-ups in the schema instead of leaving pull-up location implicit

Named physical standards:

- use `examples/qwiic_i2c_link.py` when the interface is specifically SparkFun-style Qwiic with `GND`, `3V3`, `SDA`, `SCL` in exact pin order
- use `examples/grove_i2c_link.py` when the interface is specifically Grove 4-pin I2C with `SCL`, `SDA`, `VCC`, `GND` in exact pin order
- do not collapse these into one “4-pin I2C connector” because the physical order is part of the contract

Primary sources used for the named I2C connector defaults:

- SparkFun Qwiic documentation describing the 4-pin JST-SH order as `GND`, `3.3V`, `SDA`, `SCL`
- Seeed Grove documentation describing the 4-pin I2C order as `SCL`, `SDA`, `VCC`, `GND`

## 1-Wire

The local helper uses:

- `VCC`
- `DQ`
- `GND`

Use this for:

- simple low-speed device links
- deterministic interface drawings where a single bidirectional data line matters more than the exact board connector

## MDIO

The local helper uses:

- `VCC`
- `MDC`
- `MDIO`
- `GND`

Use this for:

- MAC-to-PHY management links
- low-distance board or backplane control interfaces

## RS-422

The local helper uses a full-duplex differential model:

- `TX_P`, `TX_N`
- `RX_P`, `RX_N`
- `GND`
- `SHIELD`

Design rule:

- validate the crossover explicitly: local transmit must land on remote receive
- keep shield handling explicit in the contract instead of leaving it to assembly notes
- for the local policy schema, make drain-bond ownership explicit on one end and require paired receiver termination at both nodes

## RS-485

The local helper uses a 2-wire differential model:

- `A`
- `B`
- `GND`
- `SHIELD`

Design rule:

- treat shield policy and reference ground as part of the interface contract
- the bundled example models one validated bus segment, not full multidrop termination policy

Multidrop rule:

- use `examples/rs485_multidrop.py` when you need one biased controller end, one terminated far end, and zero-or-more unterminated drops
- make drain handling explicit as `bonded`, `pass`, or `floating` instead of burying it in cable notes

## SpaceWire

The local helper treats one full-duplex SpaceWire link as:

- transmit data differential pair: `TXD_P`, `TXD_N`
- transmit strobe differential pair: `TXS_P`, `TXS_N`
- receive data differential pair: `RXD_P`, `RXD_N`
- receive strobe differential pair: `RXS_P`, `RXS_N`
- `GND`

Primary source used for the local defaults:

- SpaceWire references describing two differential pairs in each direction, carrying data and strobe, for a total of eight signal wires per bidirectional link

## Ethernet

The local helper uses the existing T568B logical ordering:

- `TX+`
- `TX-`
- `RX+`
- `BI1+`
- `BI1-`
- `RX-`
- `BI2+`
- `BI2-`

Use this when the drawing needs a deterministic logical link or breakout, not a full PHY or magnetics schematic.

Variant rule:

- use the base Ethernet example for unshielded, non-PoE logical links
- use `examples/ethernet_poe_link.py` when shield bonding and PSE/PD role assignment are part of the contract
- model PoE role explicitly in the schema even when the drawing stays at the logical pair level
- the local Ethernet endpoint schema now also enforces exact RJ45 T568B pin numbering, not just pair naming

## PPS

The local helper uses:

- `VCC`
- `PPS`
- `GND`

Use this for:

- timing distribution notes
- simple ICD or synchronization links
- GPSDO / timing-base connections

## Design Rule

For protocol diagrams, the smallest useful contract is:

- protocol signal set is explicit
- optional signals are named as optional
- mapping is schema-checked
- rendering is generated from the validated mapping

If a connector is known but the protocol is not, start from the connector helper.

If the protocol is known but the physical connector may vary, start from the protocol helper.

If the interface is multidrop or policy-heavy, start from the policy-backed example rather than forcing it into a point-to-point harness template.
