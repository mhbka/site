FROM node:22-bookworm-slim AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM node:22-bookworm-slim AS runtime

WORKDIR /app
ENV NODE_ENV=production

COPY --from=builder /app/dist ./dist

RUN useradd --system --create-home app \
    && chown -R app:app /app

USER app
EXPOSE 4321

CMD ["node", "./dist/server/entry.mjs"]
