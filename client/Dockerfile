# Build stage
FROM docker.io/oven/bun:1-alpine as builder

WORKDIR /app

# Copy package files
COPY package.json bun.lock* ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy application files
COPY . .

# Build the application
# Use VITE_API_URL as that's what the client code expects
# Default to port-forwarded URL (8080) instead of NodePort (30080)
ARG VITE_API_URL=http://localhost:8080
ENV VITE_API_URL=${VITE_API_URL}
RUN echo "Building with API URL: $VITE_API_URL" && bun run build

# Production stage
FROM docker.io/oven/bun:1-alpine

WORKDIR /app

# Copy built application and dependencies
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./
COPY --from=builder /app/node_modules ./node_modules

# Expose port
EXPOSE 3000

# Run the application
CMD ["bun", "run", "build/index.js"]
