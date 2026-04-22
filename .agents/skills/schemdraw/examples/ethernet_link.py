import schemdraw

from helpers.pinmap import ConnectionDef, connect_by_signal
from helpers.protocols import ETHERNET_T568B_SIGNALS, ethernet_rj45, ethernet_schema, signal_pin_defs
from helpers.schema import validate_harness_schema


CONNECTIONS = [
    ConnectionDef("TX+", label="TX+", loc="top"),
    ConnectionDef("TX-", label="TX-", loc="top"),
    ConnectionDef("RX+", label="RX+", loc="top"),
    ConnectionDef("BI1+", label="BI1+", loc="top"),
    ConnectionDef("BI1-", label="BI1-", loc="bottom"),
    ConnectionDef("RX-", label="RX-", loc="bottom"),
    ConnectionDef("BI2+", label="BI2+", loc="bottom"),
    ConnectionDef("BI2-", label="BI2-", loc="bottom"),
]


def build():
    schemdraw.use("svg")
    d = schemdraw.Drawing()
    d.config(unit=2.0, fontsize=9)
    left_pins = signal_pin_defs(ETHERNET_T568B_SIGNALS, side="right")
    right_pins = signal_pin_defs(ETHERNET_T568B_SIGNALS, side="left")
    validate_harness_schema("ETH A", left_pins, "ETH B", right_pins, CONNECTIONS, ethernet_schema())
    with d:
        a = ethernet_rj45("ETH A", side="right")
        b = ethernet_rj45("ETH B", at=(10, 0), side="left")
        connect_by_signal(a, b, CONNECTIONS)
    return d


if __name__ == "__main__":
    build().save("ethernet_link.svg")
