FROM node:20-slim

# Install chromium deps for Lighthouse/puppeteer
RUN apt-get update && apt-get install -y \
  chromium \
  chromium-driver \
  ca-certificates \
  --no-install-recommends \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy server package files
COPY browser-tools-server/package*.json ./browser-tools-server/

# Install deps
RUN cd browser-tools-server && npm ci --omit=dev 2>/dev/null || npm install --omit=dev

# Copy source
COPY browser-tools-server/ ./browser-tools-server/

# Build TypeScript
RUN cd browser-tools-server && npx tsc

WORKDIR /app/browser-tools-server

# Railway sets PORT automatically
ENV PORT=3000
ENV CHROME_PATH=/usr/bin/chromium
ENV PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true
ENV PUPPETEER_EXECUTABLE_PATH=/usr/bin/chromium

EXPOSE 3000

CMD ["node", "dist/browser-connector.js"]
