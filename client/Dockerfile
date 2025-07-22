# Build stage
FROM oven/bun:1-alpine as builder

WORKDIR /app

# Copy package files
COPY package.json bun.lock* ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy application files
COPY . .

# Build the application
ENV PUBLIC_API_URL=http://localhost:30080
RUN bun run build

# Production stage
FROM oven/bun:1-alpine

WORKDIR /app

# Copy built application and dependencies
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./
COPY --from=builder /app/node_modules ./node_modules

# Expose port
EXPOSE 3000

# Run the application
CMD ["bun", "run", "build/index.js"]