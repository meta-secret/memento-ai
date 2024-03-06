APP_NAME := nervo_bot_r2d2
CORE_DIR := nervo_core
APP_DIR := nervo_bot_app

app_clean:
	cd ${CORE_DIR} && rm -rf target
	cd ${CORE_DIR} && rm -rf Cargo.lock

	cd ${APP_DIR} && rm -rf target
	cd ${APP_DIR} && rm -rf Cargo.lock

app_build:
	cd ${APP_DIR} && cargo build --release

docker_build: app_clean
	#docker build -t ${APP_NAME}:latest --progress=plain .  2>&1 | tee docker-build.log
	docker build -f Probiot.dockerfile -t ${APP_NAME}:latest .

docker_run: docker_build
	docker run -ti --name ${APP_NAME} ${APP_NAME}:latest

docker_run_daemon: docker_build
	docker run -d --name ${APP_NAME} ${APP_NAME}:latest

docker_kill:
	docker kill ${APP_NAME} || true

docker_rm:
	docker container rm ${APP_NAME} || true

docker_clean_up: docker_kill docker_rm
