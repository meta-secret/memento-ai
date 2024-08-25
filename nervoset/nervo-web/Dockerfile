FROM node:22.7-bookworm

WORKDIR /nerwo-web

COPY package.json /nerwo-web

RUN npm install

COPY package.json /nerwo-web

CMD npm run local
