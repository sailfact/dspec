FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY server.py .

# The server reads all config from env (see .env.example)
EXPOSE 8000

# Run via uvicorn; honours MCP_HOST / MCP_PORT defaults in server.py
CMD ["python", "server.py"]