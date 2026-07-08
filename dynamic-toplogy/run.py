from nexosim import Simulation
from nexosim.types import enumclass, tuple_type
@enumclass
class Topology:
    class AB(tuple_type(float, float)): ...
    class Many(tuple_type(int, float)): ...

    type = AB | Many


with Simulation("localhost:41633") as sim:

    print("\nInitializing the simulation with the `AB` topology.")

    cfg = Topology.AB(100.0, 500.0)

    sim.build(cfg)
    sim.init()

    print("\nSetting voltage to 12 V.\n")
    sim.process_event("ps_voltage", 12.0)

    print(f"Power Supply's power: {sim.try_read_events('ps_power')[0]:.2f}W")
    print(f"Load A's power: {sim.try_read_events('power_A')[0]:.2f}W")
    print(f"Load B's power: {sim.try_read_events('power_B')[0]:.2f}W")

    print("\nDropping voltage to 10 V.\n")

    sim.process_event("ps_voltage", 10.0)

    print(f"Power Supply's power: {sim.try_read_events('ps_power')[0]:.2f}W")
    print(f"Load A's power: {sim.try_read_events('power_A')[0]:.2f}W")
    print(f"Load B's power: {sim.try_read_events('power_B')[0]:.2f}W")

    print("\nReinitializing the simulation with the `Many` topology.")

    sim.terminate()

    cfg = Topology.Many(5, 100.0)

    sim.build(cfg)
    sim.init()

    print("\nSetting voltage to 12 V.\n")
    sim.process_event("ps_voltage", 12.0)

    print(f"Power supply's power: {sim.try_read_events('ps_power')[0]:.2f}W")
    print(
        "Each load's power:",
        ", ".join([f"{p:.2f}W" for p in sim.try_read_events("load_power")]),
    )

    print("\nDropping voltage to 10 V.\n")

    sim.process_event("ps_voltage", 10.0)

    print(f"Power supply's power: {sim.try_read_events('ps_power')[0]:.2f}W")
    print(
        "Each load's power:",
        ", ".join([f"{p:.2f}W" for p in sim.try_read_events("load_power")]),
    )
