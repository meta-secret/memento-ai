
app_clean:
	cd nervo_bot_app && rm -rf target
	cd nervo_bot_app && rm -rf Cargo.lock

app_build:
	cd nervo_bot_app && cargo build --release

docker_build: app_clean
	docker build -t nervo_bot:latest .

run: docker_build
	docker run -ti --name nervo_bot nervo_bot:latest

run_docker_daemon: docker_build
	docker run -d --name nervo_bot nervo_bot:latest

docker_clean:
	docker container rm nervo_bot || true
