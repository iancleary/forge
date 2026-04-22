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
- `examples/spi_peripheral.py`: clock, chip-enable, MOSI, MISO, power, and ground
- `examples/spacewire_link.py`: bidirectional data/strobe differential pairs and ground
- `examples/ethernet_link.py`: RJ45 T568B logical link
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
