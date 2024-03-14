#### Nervoset build

- [Install rust](https://www.rust-lang.org/tools/install)
- install just: `cargo install just`

- install docker: [docker](https://docs.docker.com/get-docker/)
- install docker-compose: [docker-compose](https://docs.docker.com/compose/install/linux/)

- change directory to `nervoset`: cd nervoset
- bring your configs: see examples in the `config` directory (names has to be without `.example` suffix).
  Be careful with container names in `config/docker_compose/nervoset.env`, don't use `prod` names, 
  like `nervoset/r2d2:latest`, ALWAYS add your suffix, like `nervoset/r2d2:latest` -> `nervoset/r2d2_cypherkitty_experiment:latest` 

- automation: 
    - build r2d2: `just clean docker_r2d2_build`
    - run r2d2: `just clean docker_r2d2_run`
    - run r2d2 as daemon: `just clean docker_r2d2_run_daemon`

    - build probiot_t1000: `just docker_probiot_t1000_build`
    - run probiot_t1000: `just docker_probiot_t1000_run`
    - run probiot_t1000 as daemon: `just clean docker_probiot_t1000_run_daemon`
- testing: 
    - run tests: `just clean test`
