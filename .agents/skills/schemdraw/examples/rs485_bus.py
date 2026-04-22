import schemdraw

from helpers.pinmap import ConnectionDef, connect_by_signal
from helpers.protocols import RS485_2W_SIGNALS, rs485_2w_endpoint, rs485_2w_schema, signal_pin_defs
from helpers.schema import validate_harness_schema


CONNECTIONS = [
    ConnectionDef("A", label="A", loc="top"),
    ConnectionDef("B", label="B", loc="top"),
    ConnectionDef("GND", label="GND", loc="bottom"),
    ConnectionDef("SHIELD", label="SHIELD", loc="bottom"),
]


def build():
    schemdraw.use("svg")
    d = schemdraw.Drawing()
    d.config(unit=2.0, fontsize=9)
    left_pins = signal_pin_defs(RS485_2W_SIGNALS, side="right")
    right_pins = signal_pin_defs(RS485_2W_SIGNALS, side="left")
    validate_harness_schema("NODE A", left_pins, "NODE B", right_pins, CONNECTIONS, rs485_2w_schema())
    with d:
        node_a = rs485_2w_endpoint("NODE A", side="right")
        node_b = rs485_2w_endpoint("NODE B", at=(8, 0), side="left")
        connect_by_signal(node_a, node_b, CONNECTIONS)
    return d


if __name__ == "__main__":
    build().save("rs485_bus.svg")
