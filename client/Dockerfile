FROM node:17-bullseye-slim

WORKDIR /app
COPY package.json .

RUN npm install \
    # The process is not run as root, and there are errors creating tmp dirs/files \
    # So, changing ownership to the node user.
    && chown -R node:node /app/node_modules

COPY . .

# Run as non root.
USER node

CMD ./node_modules/vite/bin/vite.js --host "0.0.0.0" --port 3000 .
