
app_clean:
	cd nervo_core && rm -rf target
	cd nervo_core && rm -rf Cargo.lock

	cd probiot && rm -rf target
	cd probiot && rm -rf Cargo.lock

app_build:
	cd probiot && cargo build --release

docker_build: app_clean
	#docker build -t nervo_bot:latest --progress=plain .  2>&1 | tee docker-build.log
	docker build -f Probiot.dockerfile -t probiot_t1000:latest .

docker_run: docker_build
	docker run -ti --name probiot_t1000 probiot_t1000:latest

docker_run_daemon: docker_build
	docker run -d --name probiot_t1000 probiot_t1000:latest

docker_kill:
	docker kill probiot_t1000 || true

docker_rm:
	docker container rm probiot_t1000 || true

docker_clean_up: docker_kill docker_rm