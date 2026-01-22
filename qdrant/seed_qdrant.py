import json
import time
import os
from qdrant_client import QdrantClient
from qdrant_client.models import PointStruct, VectorParams, Distance
from qdrant_client.http.exceptions import UnexpectedResponse

API_KEY = os.environ.get("QDRANT_API_KEY")

# Connect using the service name and the API key
client = QdrantClient(
    host="qdrant",
    port=6333,
    api_key=API_KEY,
    https=False,              # Forces plain HTTP
    timeout=60
)

COLLECTION_NAME = "nt-cctv-vehicles"

def seed_data():
    # Robust connection retry logic
    print("Attempting to connect to Qdrant...")
    connected = False
    while not connected:
        try:
            client.get_collections()
            connected = True
            print("Successfully connected to Qdrant!")
        except Exception as e:
            print(f"Connection failed: {e}. Retrying in 3 seconds...")
            time.sleep(3)

    # 1. Load JSON (ensure data.json is in the same folder as this script)
    with open('data.json', 'r') as f:
        data = json.load(f)

    # 2. Setup Collection
    vector_size = len(data[0]['vectors'])
    if not client.collection_exists(COLLECTION_NAME):
        client.create_collection(
            collection_name=COLLECTION_NAME,
            vectors_config=VectorParams(size=vector_size, distance=Distance.COSINE),
        )

    # 3. Batch Upload
    points = [
        PointStruct(
            id=item.pop("id"),
            vector=item.pop("vectors"),
            payload=item
        ) for item in data
    ]

    client.upsert(collection_name=COLLECTION_NAME, points=points)
    print(f"Done! Inserted {len(points)} points.")

if __name__ == "__main__":
    seed_data()
