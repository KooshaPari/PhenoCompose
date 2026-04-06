# nvms_ml - ML/AI Integration Layer
#
# Current: Python/NumPy stub
# Future:   Mojo when stable (mojoc/python interop)
#
# GPU Support:
#   - CUDA:  NVIDIA GPUs (torch, cupy)
#   - ROCm:  AMD GPUs (torch-rocm, cupy-rocm)
#   - Metal: Apple Silicon (torch-metal, mlx)

from typing import Optional, List, Any
import numpy as np

# ---------------------------------------------------------------------------
# GPU Backend Selection
# ---------------------------------------------------------------------------

GPU_BACKEND_AUTO = "auto"
GPU_BACKEND_CPU = "cpu"
GPU_BACKEND_CUDA = "cuda"
GPU_BACKEND_ROCM = "rocm"
GPU_BACKEND_METAL = "metal"
GPU_BACKEND_MPS = "mps"  # Apple Silicon MPS


def get_gpu_backend() -> str:
    """Detect best available GPU backend."""
    import torch

    if torch.cuda.is_available():
        return GPU_BACKEND_CUDA
    if hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
        return GPU_BACKEND_MPS
    # ROCm detection
    try:
        import torch.distributed as dist

        if dist.is_available() and hasattr(dist, "is_rocm_supported"):
            return GPU_BACKEND_ROCM
    except Exception:
        pass
    return GPU_BACKEND_CPU


def device_count() -> int:
    """Return number of available GPU devices."""
    import torch

    if torch.cuda.is_available():
        return torch.cuda.device_count()
    if hasattr(torch.backends, "mps"):
        return 1 if torch.backends.mps.is_available() else 0
    return 0


# ---------------------------------------------------------------------------
# Vector Embeddings
# ---------------------------------------------------------------------------


class VectorEmbedding:
    """Vector embedding storage with GPU acceleration."""

    def __init__(
        self,
        dim: int,
        gpu_backend: str = GPU_BACKEND_AUTO,
        max_batch_size: int = 256,
    ):
        self.dim = dim
        self.max_batch_size = max_batch_size

        # Select backend
        if gpu_backend == GPU_BACKEND_AUTO:
            gpu_backend = get_gpu_backend()

        self.gpu_backend = gpu_backend

        # Initialize storage
        self.vectors: Optional[np.ndarray] = None

        # GPU tensor (lazy init)
        self._gpu_tensor: Any = None
        self._gpu_lib: Any = None

    def _init_gpu(self):
        """Initialize GPU resources."""
        if self.gpu_backend == GPU_BACKEND_CPU or self._gpu_tensor is not None:
            return

        if self.gpu_backend == GPU_BACKEND_CUDA:
            import torch

            self._gpu_lib = torch
            self._gpu_tensor = torch.cuda.FloatTensor
        elif self.gpu_backend in (GPU_BACKEND_MPS, GPU_BACKEND_METAL):
            import torch

            self._gpu_lib = torch
            self._gpu_tensor = torch.mps.FloatTensor
        elif self.gpu_backend == GPU_BACKEND_ROCM:
            import torch

            self._gpu_lib = torch
            self._gpu_tensor = torch.cuda.FloatTensor  # ROCm uses same API

    def add(self, text: str, vector: np.ndarray) -> int:
        """Add embedding to storage."""
        if vector.shape != (self.dim,):
            raise ValueError(f"Expected shape ({self.dim},), got {vector.shape}")

        if self.vectors is None:
            self.vectors = vector.reshape(1, -1)
            return 0

        self.vectors = np.vstack([self.vectors, vector.reshape(1, -1)])
        return len(self.vectors) - 1

    def search(self, query: np.ndarray, k: int = 5) -> tuple:
        """
        Find k nearest neighbors using cosine similarity.

        Returns:
            indices: Indices of k nearest neighbors
            scores: Similarity scores
        """
        if self.vectors is None or len(self.vectors) == 0:
            return np.array([]), np.array([])

        # Normalize
        query_norm = query / (np.linalg.norm(query) + 1e-8)
        vectors_norm = self.vectors / (
            np.linalg.norm(self.vectors, axis=1, keepdims=True) + 1e-8
        )

        # Cosine similarity
        scores = np.dot(vectors_norm, query_norm)

        # Top k
        if k >= len(scores):
            indices = np.argsort(scores)[::-1]
            return indices, scores[indices]

        top_k = np.argpartition(scores, -k)[-k:]
        top_k = top_k[np.argsort(scores[top_k])[::-1]]
        return top_k, scores[top_k]

    def search_gpu(self, query: np.ndarray, k: int = 5) -> tuple:
        """
        GPU-accelerated search using PyTorch.

        Falls back to CPU if GPU unavailable.
        """
        import torch

        self._init_gpu()

        query_t = torch.from_numpy(query).float()
        vectors_t = torch.from_numpy(self.vectors).float()

        # Move to GPU if available
        if self.gpu_backend != GPU_BACKEND_CPU:
            device = (
                torch.device("cuda")
                if self.gpu_backend == GPU_BACKEND_CUDA
                else torch.device("mps")
            )
            query_t = query_t.to(device)
            vectors_t = vectors_t.to(device)

        # Cosine similarity on GPU
        query_norm = query_t / (query_t.norm() + 1e-8)
        vectors_norm = vectors_t / (vectors_t.norm(dim=1, keepdim=True) + 1e-8)
        scores = torch.mm(vectors_norm.unsqueeze(0), query_norm.unsqueeze(1)).squeeze()

        # Top k
        if k >= len(scores):
            scores, indices = scores.sort(descending=True)
            return indices.cpu().numpy(), scores.cpu().numpy()

        scores, indices = torch.topk(scores, k)
        return indices.cpu().numpy(), scores.cpu().numpy()


# ---------------------------------------------------------------------------
# Text Classification
# ---------------------------------------------------------------------------


class TextClassifier:
    """Text classifier with GPU acceleration."""

    def __init__(self, model_name: str = "distilbert", gpu_backend: str = GPU_BACKEND_AUTO):
        self.model_name = model_name
        self.gpu_backend = gpu_backend if gpu_backend != GPU_BACKEND_AUTO else get_gpu_backend()
        self.model: Any = None
        self.tokenizer: Any = None
        self._device: Any = None

    def load(self):
        """Load model and tokenizer."""
        import torch
        from transformers import AutoModelForSequenceClassification, AutoTokenizer

        self.tokenizer = AutoTokenizer.from_pretrained(self.model_name)

        # Select device
        if self.gpu_backend == GPU_BACKEND_CUDA:
            self._device = torch.device("cuda")
        elif self.gpu_backend in (GPU_BACKEND_MPS, GPU_BACKEND_METAL):
            self._device = torch.device("mps")
        elif self.gpu_backend == GPU_BACKEND_ROCM:
            self._device = torch.device("cuda")  # ROCm compatible
        else:
            self._device = torch.device("cpu")

        self.model = AutoModelForSequenceClassification.from_pretrained(
            self.model_name
        ).to(self._device)

    def predict(self, texts: List[str]) -> List[int]:
        """Predict class labels for texts."""
        if self.model is None:
            self.load()

        import torch

        # Tokenize
        inputs = self.tokenizer(
            texts, padding=True, truncation=True, return_tensors="pt"
        ).to(self._device)

        # Predict
        with torch.no_grad():
            outputs = self.model(**inputs)
            predictions = torch.argmax(outputs.logits, dim=-1)

        return predictions.cpu().tolist()

    def predict_proba(self, texts: List[str]) -> np.ndarray:
        """Predict class probabilities for texts."""
        if self.model is None:
            self.load()

        import torch

        # Tokenize
        inputs = self.tokenizer(
            texts, padding=True, truncation=True, return_tensors="pt"
        ).to(self._device)

        # Predict probabilities
        with torch.no_grad():
            outputs = self.model(**inputs)
            probs = torch.softmax(outputs.logits, dim=-1)

        return probs.cpu().numpy()


# ---------------------------------------------------------------------------
# ML Inference Server
# ---------------------------------------------------------------------------


class MLInferenceServer:
    """
    ML inference server with multi-GPU support.

    Supports:
    - CUDA (NVIDIA)
    - ROCm (AMD)
    - Metal (Apple Silicon)
    - CPU fallback
    """

    def __init__(
        self,
        model_path: str,
        gpu_backend: str = GPU_BACKEND_AUTO,
        tensor_parallel: bool = True,
    ):
        self.model_path = model_path
        self.gpu_backend = gpu_backend if gpu_backend != GPU_BACKEND_AUTO else get_gpu_backend()
        self.tensor_parallel = tensor_parallel
        self.model: Any = None
        self._world_size = 1
        self._rank = 0

    def load(self):
        """Load model with appropriate backend."""
        import torch

        # Multi-GPU setup
        if self.tensor_parallel and self.gpu_backend != GPU_BACKEND_CPU:
            self._world_size = min(device_count(), torch.get_num_threads())
        else:
            self._world_size = 1
            self._rank = 0

        # Device selection
        if self.gpu_backend == GPU_BACKEND_CUDA:
            device = torch.device(f"cuda:{self._rank}")
        elif self.gpu_backend in (GPU_BACKEND_MPS, GPU_BACKEND_METAL):
            device = torch.device("mps")
        elif self.gpu_backend == GPU_BACKEND_ROCM:
            device = torch.device("cuda")  # ROCm compatible
        else:
            device = torch.device("cpu")

        # Load model (placeholder - replace with actual model loading)
        # self.model = load_your_model(self.model_path, device=device)
        self._device = device

    def infer(self, inputs: np.ndarray) -> np.ndarray:
        """Run inference."""
        if self.model is None:
            self.load()

        import torch

        # Convert to tensor
        if isinstance(inputs, np.ndarray):
            tensor = torch.from_numpy(inputs).float().to(self._device)
        else:
            tensor = inputs

        # Inference
        with torch.no_grad():
            outputs = self.model(tensor)

        return outputs.cpu().numpy()


# ---------------------------------------------------------------------------
# Apple Silicon (M-series) Optimizations
# ---------------------------------------------------------------------------


class MetalOptimizer:
    """
    Apple Silicon (M1/M2/M3) specific optimizations using:
    - Metal Performance Shaders (MPS)
    - MLX (Apple's ML framework)
    - NEON SIMD
    """

    @staticmethod
    def is_metal_available() -> bool:
        """Check if Metal GPU is available."""
        import torch

        return hasattr(torch.backends, "mps") and torch.backends.mps.is_available()

    @staticmethod
    def optimize_for_metal():
        """Apply Apple Silicon specific optimizations."""
        import torch

        if MetalOptimizer.is_metal_available():
            # Enable MPS fallbacks for operations not natively supported
            torch.backends.mps.enable_fallback_to_cpu()

    @staticmethod
    def get_metal_device():
        """Get Metal device for PyTorch."""
        import torch

        if MetalOptimizer.is_metal_available():
            return torch.device("mps")
        return torch.device("cpu")


class CUDAOptimizer:
    """NVIDIA CUDA optimizations."""

    @staticmethod
    def is_cuda_available() -> bool:
        import torch

        return torch.cuda.is_available()

    @staticmethod
    def get_cuda_device(index: int = 0):
        """Get CUDA device."""
        import torch

        if CUDAOptimizer.is_cuda_available():
            return torch.device(f"cuda:{index}")
        return torch.device("cpu")

    @staticmethod
    def optimize_for_cuda():
        """Apply CUDA-specific optimizations."""
        import torch

        if CUDAOptimizer.is_cuda_available():
            # Enable cuDNN auto-tuner
            torch.backends.cudnn.benchmark = True
            # Enable TF32 for Ampere+
            torch.backends.cuda.matmul.allow_tf32 = True
            torch.backends.cudnn.allow_tf32 = True


class ROCmOptimizer:
    """AMD ROCm optimizations."""

    @staticmethod
    def is_rocm_available() -> bool:
        """Check if ROCm is available."""
        try:
            import torch

            return hasattr(torch.version, "hip") and torch.version.hip is not None
        except Exception:
            return False

    @staticmethod
    def get_rocm_device(index: int = 0):
        """Get ROCm device."""
        if ROCmOptimizer.is_rocm_available():
            import torch

            return torch.device(f"cuda:{index}")  # ROCm uses same API
        return torch.device("cpu")


# ---------------------------------------------------------------------------
# Auto-selection
# ---------------------------------------------------------------------------


def create_optimizer():
    """Create optimal optimizer based on available hardware."""
    if CUDAOptimizer.is_cuda_available():
        return CUDAOptimizer()
    if MetalOptimizer.is_metal_available():
        return MetalOptimizer()
    if ROCmOptimizer.is_rocm_available():
        return ROCmOptimizer()
    return None  # CPU only


__all__ = [
    # Backends
    "get_gpu_backend",
    "device_count",
    "GPU_BACKEND_AUTO",
    "GPU_BACKEND_CPU",
    "GPU_BACKEND_CUDA",
    "GPU_BACKEND_ROCM",
    "GPU_BACKEND_METAL",
    "GPU_BACKEND_MPS",
    # Classes
    "VectorEmbedding",
    "TextClassifier",
    "MLInferenceServer",
    "MetalOptimizer",
    "CUDAOptimizer",
    "ROCmOptimizer",
    "create_optimizer",
]
