from nexosim import Simulation
from nexosim.time import MonotonicTime
from pprint import pprint


with Simulation("0.0.0.0:41633") as sim:
    sim.build()
    sim.init()

    sim.step_until(MonotonicTime(1))
    pprint(sim.try_read_events("yaw", tuple[MonotonicTime, float]))