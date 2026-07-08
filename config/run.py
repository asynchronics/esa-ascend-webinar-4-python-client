from nexosim import Simulation


with open("config/gyro_config.toml", "r") as file:
    gyro_cfg = file.read()
with open("config/rw_config.toml", "r") as file:
    rw_cfg = file.read()

cfg = (gyro_cfg, rw_cfg)

with Simulation("localhost:41633") as sim:
    sim.build(cfg)
