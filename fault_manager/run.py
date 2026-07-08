from nexosim import Simulation
from nexosim.time import Duration, MonotonicTime

with Simulation("localhost:41633") as sim:
    sim.build()
    sim.init()

    # Make the OBC periodically send a Ping TC.
    sim.schedule_event(Duration(1), "obc_send_tc", bytes("ping", "utf8"), Duration(1))

    # Disable the connection after 2 seconds.
    sim.schedule_event(MonotonicTime(2), "toggle_tmtc_fault", True)

    # Advance the simulation by 5 seconds.
    sim.step_until(MonotonicTime(5))
