from __future__ import annotations

from helpers.pinmap import PinDef, endpoint
from helpers.schema import EndpointSchema, HarnessSchema


def dsub9(label: str, signals: list[str], *, at: tuple[float, float] | None = None, side: str = "left"):
    if len(signals) != 9:
        raise ValueError("dsub9 requires exactly 9 signals")
    pins = [PinDef(signal, str(index + 1), side=side) for index, signal in enumerate(signals)]
    return endpoint(f"{label}\nDE-9", pins, at=at)


def dsub15(label: str, signals: list[str], *, at: tuple[float, float] | None = None, side: str = "left"):
    if len(signals) != 15:
        raise ValueError("dsub15 requires exactly 15 signals")
    pins = [PinDef(signal, str(index + 1), side=side) for index, signal in enumerate(signals)]
    return endpoint(f"{label}\nDA-15", pins, at=at)


def dsub25(label: str, signals: list[str], *, at: tuple[float, float] | None = None, side: str = "left"):
    if len(signals) != 25:
        raise ValueError("dsub25 requires exactly 25 signals")
    pins = [PinDef(signal, str(index + 1), side=side) for index, signal in enumerate(signals)]
    return endpoint(f"{label}\nDB-25", pins, at=at)


def header_1x(label: str, signals: list[str], *, at: tuple[float, float] | None = None, side: str = "left"):
    pins = [PinDef(signal, str(index + 1), side=side) for index, signal in enumerate(signals)]
    return endpoint(f"{label}\n1x{len(signals)} Header", pins, at=at)


def header_2x(
    label: str,
    left_signals: list[str],
    right_signals: list[str],
    *,
    at: tuple[float, float] | None = None,
):
    if len(left_signals) != len(right_signals):
        raise ValueError("header_2x requires the same number of left and right signals")
    pins = [
        *(PinDef(signal, str(index * 2 + 1), side="left") for index, signal in enumerate(left_signals)),
        *(PinDef(signal, str(index * 2 + 2), side="right") for index, signal in enumerate(right_signals)),
    ]
    return endpoint(f"{label}\n2x{len(left_signals)} Header", pins, at=at)


def terminal_block(label: str, signals: list[str], *, at: tuple[float, float] | None = None, side: str = "left"):
    pins = [PinDef(signal, str(index + 1), side=side) for index, signal in enumerate(signals)]
    return endpoint(f"{label}\nTerminal Block", pins, at=at)


DE9_RS232_SIGNALS = (
    "DCD",
    "RXD",
    "TXD",
    "DTR",
    "GND",
    "DSR",
    "RTS",
    "CTS",
    "RI",
)

RJ45_T568B_SIGNALS = (
    "TX+",
    "TX-",
    "RX+",
    "BI1+",
    "BI1-",
    "RX-",
    "BI2+",
    "BI2-",
)

RS232_ALIASES = {
    "SHLD": "SHIELD",
    "SHLDGND": "SHIELD",
}


def rs232_de9_schema() -> EndpointSchema:
    return EndpointSchema(
        family="DE-9 RS-232",
        pin_count=9,
        exact_signals=DE9_RS232_SIGNALS,
    )


def rs232_terminal_block_schema() -> EndpointSchema:
    return EndpointSchema(
        family="Terminal Block RS-232 Breakout",
        pin_count=9,
        exact_signals=DE9_RS232_SIGNALS,
    )


def rs232_breakout_schema() -> HarnessSchema:
    return HarnessSchema(
        name="RS-232 DE-9 breakout",
        left=rs232_de9_schema(),
        right=rs232_terminal_block_schema(),
        required_connections=tuple((signal, signal) for signal in DE9_RS232_SIGNALS),
        aliases=RS232_ALIASES,
    )


def rs232_de9(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    return dsub9(label, list(DE9_RS232_SIGNALS), at=at, side=side)


def rj45_t568b(label: str, *, at: tuple[float, float] | None = None, side: str = "left"):
    pins = [PinDef(signal, str(index + 1), side=side) for index, signal in enumerate(RJ45_T568B_SIGNALS)]
    return endpoint(f"{label}\nRJ45 T568B", pins, at=at)
