# Docker Configuration Update Summary

## Changes Made

### 1. Updated `docker-compose.yml`

Added all missing environment variables to the backend service:

#### ✅ Added Environment Variables:
- **CCTV_AUTH_TOKEN**: Authentication token for CCTV API (loaded from `.env` file)
- **CCTV_API_URL**: CCTV metadata API endpoint
- **CCTV_ID**: CCTV camera identifier
- **AI_SERVICE_URL**: Set to `http://host.docker.internal:5090` (was empty)

#### 📋 Complete Environment Configuration:
```yaml
environment:
  # Qdrant Vector Database Configuration
  - QDRANT_URL=http://qdrant:6334
  - QDRANT_API_KEY=my-secret-api-key
  - COLLECTION_NAME=nt-cctv-vehicles

  # AI Service Configuration
  - AI_SERVICE_URL=http://host.docker.internal:5090

  # CCTV API Configuration
  - CCTV_API_URL=https://ntvideo.totbb.net/video-metadata/train-data-condition
  - CCTV_AUTH_TOKEN=${CCTV_AUTH_TOKEN}
  - CCTV_ID=cctv01
```

### 2. Created `.env.example`

Created a template file documenting all required environment variables:
- Organized into logical sections (Qdrant, AI Service, CCTV API)
- Includes helpful comments explaining each variable
- Shows default values where applicable
- Highlights required variables (CCTV_AUTH_TOKEN)

### 3. Updated `README.md`

Added comprehensive Docker deployment section including:
- Quick start guide with docker-compose
- Environment variable documentation
- Platform-specific instructions (Windows/Mac/Linux)
- Common Docker commands
- Dockerfile architecture explanation
- Production deployment best practices

## Next Steps

### To Use the Updated Configuration:

1. **Create your `.env` file**:
   ```bash
   cp .env.example .env
   ```

2. **Edit `.env` and add your CCTV API token**:
   ```bash
   CCTV_AUTH_TOKEN=your_actual_token_here
   ```

3. **Start the services**:
   ```bash
   docker-compose up -d
   ```

4. **View logs to verify everything is working**:
   ```bash
   docker-compose logs -f backend
   ```

### Platform-Specific Notes:

#### Windows/Mac:
- No changes needed
- `host.docker.internal` works out of the box

#### Linux:
- Change AI_SERVICE_URL in `docker-compose.yml`:
  ```yaml
  - AI_SERVICE_URL=http://172.17.0.1:5090
  ```

## Environment Variables Reference

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `QDRANT_URL` | No | `http://localhost:6334` | Qdrant database URL |
| `QDRANT_API_KEY` | No | `my-secret-api-key` | Qdrant authentication key |
| `COLLECTION_NAME` | No | `nt-cctv-vehicles` | Qdrant collection name |
| `AI_SERVICE_URL` | No | `http://localhost:5090` | AI embedding service URL |
| `CCTV_API_URL` | No | `https://ntvideo.totbb.net/...` | CCTV metadata API endpoint |
| `CCTV_AUTH_TOKEN` | **YES** | - | CCTV API authentication token |
| `CCTV_ID` | No | `cctv01` | CCTV camera identifier |

## Files Modified

1. ✅ `docker-compose.yml` - Added missing environment variables
2. ✅ `.env.example` - Created environment template
3. ✅ `README.md` - Added Docker deployment documentation

## Verification

After starting the services, you should see:

```
========================================
🚀 Starting CCTV Search Backend
   -> Qdrant URL : http://qdrant:6334
   -> AI Service : http://host.docker.internal:5090
   -> Collection : nt-cctv-vehicles
========================================
Setting up collection...
✅ Collection is ready
Creating datetime field index...
✅ Datetime field index created successfully
✅ Background scheduler started (every 10 minutes)
```

The application is now fully configured and ready to:
- Connect to Qdrant vector database
- Call the AI embedding service
- Fetch images from the CCTV API every 10 minutes
- Process and index CCTV images automatically
