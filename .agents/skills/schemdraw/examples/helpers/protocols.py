from __future__ import annotations

from helpers.connectors import header_1x, header_2x, rj45_t568b
from helpers.pinmap import PinDef, endpoint
from helpers.schema import EndpointSchema, HarnessSchema


def signal_pin_defs(signals: tuple[str, ...], *, side: str = "left") -> list[PinDef]:
    return [PinDef(signal, str(index + 1), side=side) for index, signal in enumerate(signals)]


def exact_endpoint_schema(family: str, signals: tuple[str, ...]) -> EndpointSchema:
    return EndpointSchema(
        family=family,
        pin_count=len(signals),
        exact_signals=signals,
    )


def passthrough_schema(name: str, family: str, signals: tuple[str, ...]) -> HarnessSchema:
    endpoint_schema = exact_endpoint_schema(family, signals)
    return HarnessSchema(
        name=name,
        left=endpoint_schema,
        right=endpoint_schema,
        required_connections=tuple((signal, signal) for signal in signals),
    )


def swd_signals(*, include_reset: bool = True, include_swo: bool = False) -> tuple[str, ...]:
    signals = ["VTREF", "SWDIO", "SWCLK"]
    if include_swo:
        signals.append("SWO")
    if include_reset:
        signals.append("NRST")
    signals.append("GND")
    return tuple(signals)


def swd_header(
    label: str,
    *,
    at: tuple[float, float] | None = None,
    side: str = "left",
    include_reset: bool = True,
    include_swo: bool = False,
):
    return header_1x(label, list(swd_signals(include_reset=include_reset, include_swo=include_swo)), at=at, side=side)


def swd_schema(*, include_reset: bool = True, include_swo: bool = False) -> HarnessSchema:
    signals = swd_signals(include_reset=include_reset, include_swo=include_swo)
    return passthrough_schema("SWD programming link", "SWD header", signals)


def jtag_signals(*, include_trst: bool = False, include_srst: bool = False) -> tuple[str, ...]:
    signals = ["VTREF", "TMS", "TCK", "TDI", "TDO"]
    if include_trst:
        signals.append("TRST_N")
    if include_srst:
        signals.append("SRST_N")
    signals.append("GND")
    return tuple(signals)


def jtag_header(
    label: str,
    *,
    at: tuple[float, float] | None = None,
    side: str = "left",
    include_trst: bool = False,
    include_srst: bool = False,
):
    return header_1x(label, list(jtag_signals(include_trst=include_trst, include_srst=include_srst)), at=at, side=side)


def jtag_schema(*, include_trst: bool = False, include_srst: bool = False) -> HarnessSchema:
    signals = jtag_signals(include_trst=include_trst, include_srst=include_srst)
    return passthrough_schema("JTAG programming link", "JTAG header", signals)


SPI_SIGNALS = ("VCC", "CS_N", "SCLK", "MOSI", "MISO", "GND")


def spi_header(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(SPI_SIGNALS), at=at, side=side)


def spi_schema() -> HarnessSchema:
    return passthrough_schema("SPI peripheral link", "SPI header", SPI_SIGNALS)


UART_SIGNALS = ("VCC", "TX", "RX", "GND")


def uart_header(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(UART_SIGNALS), at=at, side=side)


def uart_schema() -> HarnessSchema:
    return passthrough_schema("UART serial link", "UART header", UART_SIGNALS)


I2C_SIGNALS = ("VCC", "SCL", "SDA", "GND")


def i2c_header(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(I2C_SIGNALS), at=at, side=side)


def i2c_schema() -> HarnessSchema:
    return passthrough_schema("I2C low-speed link", "I2C header", I2C_SIGNALS)


ONEWIRE_SIGNALS = ("VCC", "DQ", "GND")


def onewire_header(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(ONEWIRE_SIGNALS), at=at, side=side)


def onewire_schema() -> HarnessSchema:
    return passthrough_schema("1-Wire short-reach link", "1-Wire header", ONEWIRE_SIGNALS)


MDIO_SIGNALS = ("VCC", "MDC", "MDIO", "GND")


def mdio_header(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(MDIO_SIGNALS), at=at, side=side)


def mdio_schema() -> HarnessSchema:
    return passthrough_schema("MDIO management link", "MDIO header", MDIO_SIGNALS)


RS422_SIGNALS = ("TX_P", "TX_N", "RX_P", "RX_N", "GND", "SHIELD")


def rs422_endpoint(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(RS422_SIGNALS), at=at, side=side)


def rs422_schema() -> HarnessSchema:
    endpoint_schema = exact_endpoint_schema("RS-422 endpoint", RS422_SIGNALS)
    return HarnessSchema(
        name="RS-422 full-duplex link",
        left=endpoint_schema,
        right=endpoint_schema,
        required_connections=(
            ("TX_P", "RX_P"),
            ("TX_N", "RX_N"),
            ("RX_P", "TX_P"),
            ("RX_N", "TX_N"),
            ("GND", "GND"),
            ("SHIELD", "SHIELD"),
        ),
    )


RS485_2W_SIGNALS = ("A", "B", "GND", "SHIELD")


def rs485_2w_endpoint(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(RS485_2W_SIGNALS), at=at, side=side)


def rs485_2w_schema() -> HarnessSchema:
    return passthrough_schema("RS-485 2-wire bus segment", "RS-485 2-wire endpoint", RS485_2W_SIGNALS)


SPACEWIRE_SIGNALS = (
    "TXD_P",
    "TXD_N",
    "TXS_P",
    "TXS_N",
    "RXD_P",
    "RXD_N",
    "RXS_P",
    "RXS_N",
    "GND",
)


def spacewire_endpoint(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(SPACEWIRE_SIGNALS), at=at, side=side)


def spacewire_schema() -> HarnessSchema:
    endpoint_schema = exact_endpoint_schema("SpaceWire cable endpoint", SPACEWIRE_SIGNALS)
    return HarnessSchema(
        name="SpaceWire link",
        left=endpoint_schema,
        right=endpoint_schema,
        required_connections=(
            ("TXD_P", "RXD_P"),
            ("TXD_N", "RXD_N"),
            ("TXS_P", "RXS_P"),
            ("TXS_N", "RXS_N"),
            ("RXD_P", "TXD_P"),
            ("RXD_N", "TXD_N"),
            ("RXS_P", "TXS_P"),
            ("RXS_N", "TXS_N"),
            ("GND", "GND"),
        ),
    )


ETHERNET_T568B_SIGNALS = (
    "TX+",
    "TX-",
    "RX+",
    "BI1+",
    "BI1-",
    "RX-",
    "BI2+",
    "BI2-",
)


def ethernet_rj45(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return rj45_t568b(label, at=at, side=side)


def ethernet_schema() -> HarnessSchema:
    return passthrough_schema("Ethernet T568B link", "RJ45 T568B", ETHERNET_T568B_SIGNALS)


PPS_SIGNALS = ("VCC", "PPS", "GND")


def pps_header(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return header_1x(label, list(PPS_SIGNALS), at=at, side=side)


def pps_schema() -> HarnessSchema:
    return passthrough_schema("PPS timing link", "PPS header", PPS_SIGNALS)


def fpga_jtag_2x5(label: str, *, at: tuple[float, float] | None = None):
    return header_2x(
        label,
        ["VTREF", "TMS", "TDI", "TDO", "GND"],
        ["NC", "TCK", "NC", "SRST_N", "GND"],
        at=at,
    )


def generic_endpoint(label: str, signals: tuple[str, ...], *, at: tuple[float, float] | None = None, side: str = "left"):
    pins = signal_pin_defs(signals, side=side)
    return endpoint(label, pins, at=at)
