
app_clean:
	cd nervo_bot_app && rm -rf target
	cd nervo_bot_app && rm -rf Cargo.lock
	cd nervo_core && rm -rf target
	cd nervo_core && rm -rf Cargo.lock

app_build:
	cd nervo_bot_app && cargo build --release

docker_build: app_clean
	#docker build -t nervo_bot:latest --progress=plain .  2>&1 | tee docker-build.log
	docker build -t nervo_bot:latest .

docker_run: docker_build
	docker run -ti --name nervo_bot_dk nervo_bot:latest

docker_run_daemon: docker_build
	docker run -d --name nervo_bot_dk nervo_bot:latest

docker_stop:
	docker kill nervo_bot_dk || true

docker_clean:
	docker container rm nervo_bot_dk || true
