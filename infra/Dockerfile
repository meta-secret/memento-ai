FROM jetpackio/devbox:latest

ENV INFRA_DIR=/home/devbox/infra

RUN mkdir -p ${INFRA_DIR}
WORKDIR ${INFRA_DIR}

COPY devbox.json devbox.json
RUN devbox run echo "Packages installed"

COPY . ${INFRA_DIR}

CMD devbox run task
