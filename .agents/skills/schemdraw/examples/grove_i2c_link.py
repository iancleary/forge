import schemdraw

from helpers.pinmap import ConnectionDef, connect_by_signal
from helpers.protocols import GROVE_I2C_PIN_MAP, grove_i2c_header, grove_i2c_link_schema, pin_map_pin_defs
from helpers.schema import validate_harness_schema


CONNECTIONS = [
    ConnectionDef("SCL", label="SCL", loc="top"),
    ConnectionDef("SDA", label="SDA", loc="top"),
    ConnectionDef("VCC", label="VCC", loc="top"),
    ConnectionDef("GND", label="GND", loc="bottom"),
]


def build():
    schemdraw.use("svg")
    d = schemdraw.Drawing()
    d.config(unit=2.0, fontsize=10)
    left_pins = pin_map_pin_defs(GROVE_I2C_PIN_MAP, odd_side="right", even_side="right")
    right_pins = pin_map_pin_defs(GROVE_I2C_PIN_MAP, odd_side="left", even_side="left")
    validate_harness_schema("CONTROLLER", left_pins, "MODULE", right_pins, CONNECTIONS, grove_i2c_link_schema())
    with d:
        controller = grove_i2c_header("CONTROLLER", at=(0, 0), side="right")
        module = grove_i2c_header("MODULE", at=(8, 0), side="left")
        connect_by_signal(controller, module, CONNECTIONS)
    return d


if __name__ == "__main__":
    build().save("grove_i2c_link.svg")
