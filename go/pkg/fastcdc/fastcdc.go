package fastcdc

// Config defines the parameters for the FastCDC algorithm.
type Config struct {
	MinSize int
	AvgSize int
	MaxSize int
}

// DefaultConfig returns the recommended settings for general purpose deduplication.
func DefaultConfig() Config {
	return Config{
		MinSize: 8 * 1024,  // 8KB
		AvgSize: 16 * 1024, // 16KB
		MaxSize: 32 * 1024, // 32KB
	}
}

// Chunk represents a segment of data identified by the CDC process.
type Chunk struct {
	Offset int
	Length int
	Hash   uint64
}

// Chunker is the interface for stream-based content-defined chunking.
type Chunker interface {
	Next() (*Chunk, error)
}
