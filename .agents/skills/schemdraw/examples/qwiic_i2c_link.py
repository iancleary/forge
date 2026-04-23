import schemdraw

from helpers.pinmap import ConnectionDef, connect_by_signal
from helpers.protocols import QWIIC_I2C_PIN_MAP, qwiic_i2c_header, qwiic_i2c_link_schema, pin_map_pin_defs
from helpers.schema import validate_harness_schema


CONNECTIONS = [
    ConnectionDef("GND", label="GND", loc="bottom"),
    ConnectionDef("3V3", label="3V3", loc="top"),
    ConnectionDef("SDA", label="SDA", loc="top"),
    ConnectionDef("SCL", label="SCL", loc="top"),
]


def build():
    schemdraw.use("svg")
    d = schemdraw.Drawing()
    d.config(unit=2.0, fontsize=10)
    left_pins = pin_map_pin_defs(QWIIC_I2C_PIN_MAP, odd_side="right", even_side="right")
    right_pins = pin_map_pin_defs(QWIIC_I2C_PIN_MAP, odd_side="left", even_side="left")
    validate_harness_schema("HOST", left_pins, "SENSOR", right_pins, CONNECTIONS, qwiic_i2c_link_schema())
    with d:
        host = qwiic_i2c_header("HOST", at=(0, 0), side="right")
        sensor = qwiic_i2c_header("SENSOR", at=(8, 0), side="left")
        connect_by_signal(host, sensor, CONNECTIONS)
    return d


if __name__ == "__main__":
    build().save("qwiic_i2c_link.svg")
