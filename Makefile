
build:
	cargo build --release
	docker build -t nervo_bot:latest .

run: build
	docker run -ti --name nervo_bot nervo_bot:latest

run_daemon:
	docker run -d --name nervo_bot nervo_bot:latest

docker_clean:
	docker container prune
