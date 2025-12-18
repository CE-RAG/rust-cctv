# Rust CCTV Search Backend

A high-performance backend service for searching CCTV footage using vector embeddings and Qdrant vector database.

## Features

- **Image Search**: Search for CCTV images using natural language queries
- **Datetime Filtering**: Filter search results by date and time ranges
- **URL Support**: Works with both local filenames and image URLs
- **Filename Parsing**: Automatically extracts metadata from CCTV filenames
- **Automated Image Fetching**: Background scheduler that automatically fetches and indexes images from CCTV metadata API every 10 minutes

## Software Architecture
```mermaid
graph LR
    %% Nodes
    A[Client]
    B[Rust CCTV API]
    D[SigLib]
    E[Image Embedding]
    F[Text Embedding]
    C[Qdrant Vector DB]
    %% Insert Image Flow
    A -->|Insert Image| B
    B -->|Call Image Embedding| D
    D -->|Image Vector| E
    E -->|Insert Vector + Metadata| C
    %% Search Flow
    A -->|Search Text| B
    B -->|Call Text Embedding| D
    D -->|Text Vector| F
    F -->|Query Vector DB| C
    C -->|Search Results| B
    B --> A
```

## Supported Filename Formats

- Underscore format: `cctv08_2025-10-08_06-32_4.jpg`
- Dash format: `cctv08-2025-10-08-06-32-4.jpg`
- Full URLs: `https://i.postimg.cc/XNxQM7Z2/cctv08-2025-10-08-06-32-4.jpg`

## Project Structure

```
rust-cctv/
├── src/
│   ├── main.rs                      # Application entry point & scheduler
│   ├── handlers.rs                  # HTTP request handlers
│   ├── models/
│   │   ├── mod.rs
│   │   └── search.rs                # Data models & request/response types
│   └── services/
│       ├── mod.rs                   # Service module exports
│       ├── ai_service.rs            # AI embedding service integration
│       ├── cctv_api.rs              # CCTV metadata API client
│       ├── qdrant_service.rs        # Qdrant vector DB operations
│       └── filename_utils.rs        # CCTV filename parsing utilities
├── Cargo.toml
├── .env                             # Environment configuration
└── README.md
```

## Installation

1. Clone the repository
2. Install dependencies
   ```bash
   cargo build
   ```
3. Set up environment variables (see Configuration)
4. Run the service
   ```bash
   cargo run
   ```
   
   The application will automatically:
   - Create the collection if it doesn't exist
   - Set up a datetime field index for filtering

## Configuration

Configure the application using environment variables:

- `QDRANT_URL`: URL of the Qdrant vector database (default: `http://localhost:6334`)
- `AI_SERVICE_URL`: URL of the AI embedding service (default: `http://localhost:5090`)
- `COLLECTION_NAME`: Name of the Qdrant collection (default: `ntcctvvehicles`)
- `QDRANT_API_KEY`: API key for Qdrant (if required)
- `CCTV_API_URL`: URL of the CCTV metadata API (default: `https://ntvideo.totbb.net/video-metadata/train-data-condition`)
- `CCTV_AUTH_TOKEN`: Bearer token for CCTV metadata API authentication (required)
- `CCTV_ID`: CCTV camera ID to fetch images from (default: `cctv01`)

## Collection and Index Setup

The application automatically handles:
1. **Collection Creation**: Creates the collection with 768-dimensional vectors and cosine distance if it doesn't exist
2. **Datetime Index**: Creates a datetime field index to enable filtering by date and time ranges

If you're setting up manually, you would need to:
```bash
# Create index for datetime field
curl -X PUT "http://localhost:6334/collections/{collection_name}/index" \
  -H "Content-Type: application/json" \
  -d '{
    "field_name": "datetime",
    "field_schema": "datetime"
  }'
```

## Automated Image Fetching

The application includes a background scheduler that automatically fetches and indexes CCTV images from the metadata API. This feature runs independently from the web server.

### How It Works

1. **Scheduler**: Currently runs every 1 minute (configurable via cron expression in `main.rs`)
2. **Fetch Limit**: Fetches up to 20 images per run
3. **Date Range**: Queries images from the last 2 days
4. **Processing**: For each fetched image:
   - Downloads the image metadata
   - Generates vector embeddings via the AI service
   - Parses the filename to extract datetime and camera information
   - Stores the embedding and metadata in Qdrant

### Logs

The scheduler provides detailed logging:
```
⏰ Running scheduled CCTV image fetch...
📡 Fetching CCTV training data from API...
   -> CCTV ID: cctv01
   -> Date Range: 2025-12-15 08:00:00 to 2025-12-17 20:00:00
   -> Limit: 20
✅ Successfully fetched 20 images from CCTV API
📥 Processing 20 images...
   [1/20] Processing: https://example.com/image1.jpg
      ✅ Inserted successfully
...
✅ Scheduled task completed
```

### Configuration

To configure the automated fetching, update these environment variables:
- `CCTV_API_URL`: The API endpoint
- `CCTV_AUTH_TOKEN`: Your Bearer authentication token
- `CCTV_ID`: The camera ID to fetch images from

To change the schedule interval, modify the cron expression in `src/main.rs` (line ~79):
```rust
// Current: Every 1 minute
let job = Job::new_async("0 */1 * * * *", move |_uuid, _l| {
    // ... job logic
});

// Examples:
// Every 5 minutes:  "0 */5 * * * *"
// Every 10 minutes: "0 */10 * * * *"
// Every hour:       "0 0 * * * *"
// Every 30 minutes: "0 */30 * * * *"
```

The scheduler starts automatically when the application launches and runs in the background.

## API Endpoints

### Insert Image

Insert a new CCTV image with metadata.

**Endpoint**: `POST /insert_image`

**Request Body**:
```json
{
  "image": "https://i.postimg.cc/XNxQM7Z2/cctv08-2025-10-08-06-32-4.jpg"
}
```

**Response**:
```json
{
  "status": "ok",
  "point_id": 123456789,
  "type": "image_embedding",
  "embedding": [0.1, 0.2, 0.3, ...]
}
```

### Search Images

Search for images similar to a text query, optionally filtered by datetime range.

**Endpoint**: `POST /search`

**Request Body**:
```json
{
  "query": "red car speeding",
  "top_k": 5,
  "start_date": "2025-10-08T06:00:00Z",
  "end_date": "2025-10-08T07:00:00Z"
}
```

Or with empty datetime strings (no filtering):
```json
{
  "query": "red car speeding",
  "top_k": 5,
  "start_date": "",
  "end_date": ""
}
```

**Parameters**:
- `query`: Text description of what you're looking for
- `top_k`: Number of results to return (optional, default: 5)
- `start_date`: Start of datetime range in RFC 3339 format (optional, can be empty string)
- `end_date`: End of datetime range in RFC 3339 format (optional, can be empty string)

**Response**:
```json
[
  {
    "filename": "cctv08-2025-10-08-06-32-4.jpg",
    "caption": "red car speeding",
    "score": 0.89,
    "datetime": "2025-10-08T06:32:00Z"
  },
  ...
]
```

## Datetime Filtering

The search endpoint supports filtering by datetime range using RFC 3339 format:

- `2025-10-08T06:32:00Z` (RFC 3339, UTC)
- `2025-10-08T06:32:00` (without timezone, UTC assumed)
- `2025-10-08T06:32` (without timezone and seconds)
- `2025-10-08` (date only, midnight assumed)

## Dependencies

- **actix-web** (4.12.1): HTTP server framework
- **qdrant-client** (1.10): Vector database client
- **reqwest** (0.11): HTTP client for API calls
- **tokio** (1.x): Async runtime with full features
- **tokio-cron-scheduler** (0.9): Background task scheduler
- **chrono** (0.4): Datetime handling
- **chrono-tz** (0.8): Timezone support (Bangkok/Thailand)
- **serde** (1.0): Serialization/deserialization
- **serde_json** (1.0): JSON support
- **dotenv** (0.15): Environment variable management
- **rand** (0.8): Random number generation

## Example Usage

1. Start the server:
   ```bash
   cargo run
   ```

2. Insert an image:
   ```bash
   curl -X POST http://localhost:8080/insert_image \
     -H "Content-Type: application/json" \
     -d '{"image": "https://i.postimg.cc/XNxQM7Z2/cctv08-2025-10-08-06-32-4.jpg"}'
   ```

3. Search for images:
   ```bash
   curl -X POST http://localhost:8080/search \
     -H "Content-Type: application/json" \
     -d '{
       "query": "red car",
       "start_date": "2025-10-08T06:00:00Z",
       "end_date": "2025-10-08T07:00:00Z"
     }'
   ```

## Troubleshooting

### 422 Unprocessable Entity from AI Service

If you see errors like:
```
❌ Failed to get embedding: AI Image Service returned error: 422 Unprocessable Entity
```

**Cause**: The AI service is receiving image URLs but expects local file paths or base64-encoded images.

**Solutions**:
1. **Update your AI service** to accept image URLs and download them
2. **Modify the Rust code** to download images and send as base64
3. **Ensure your AI service** accepts the `image_path` field in the request

Example Python AI service fix:
```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import requests
from PIL import Image
from io import BytesIO

class PredictRequest(BaseModel):
    image_path: str = None
    text: str = None

@app.post("/predict")
async def predict(request: PredictRequest):
    if request.image_path:
        # Check if it's a URL
        if request.image_path.startswith(('http://', 'https://')):
            response = requests.get(request.image_path)
            image = Image.open(BytesIO(response.content))
        else:
            image = Image.open(request.image_path)
        
        embedding = your_model.encode_image(image)
        return {"vector": embedding.tolist()}
```

### Connection Timeout to CCTV API

If you see:
```
❌ Connection timed out - API server may be unreachable
```

**Check**:
- Network connectivity to the CCTV API server
- Firewall settings
- API URL is correct in `.env`
- API server is running and accessible

### Authentication Errors

If you see `401 Unauthorized` or `403 Forbidden`:

**Check**:
- `CCTV_AUTH_TOKEN` is set correctly in `.env`
- Token hasn't expired
- Token has proper permissions

### Qdrant Connection Issues

If the application fails to connect to Qdrant:

**Check**:
- Qdrant is running: `docker ps` or check your Qdrant instance
- `QDRANT_URL` is correct in `.env`
- `QDRANT_API_KEY` is set if authentication is enabled
- Network connectivity to Qdrant server

## License

This project is licensed under the MIT License.
