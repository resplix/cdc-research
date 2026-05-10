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

// FastCDC implements the Chunker interface using the FastCDC algorithm.
type FastCDC struct {
	data   []byte
	pos    int
	config Config
	maskS  uint64 // Small mask for normalization
	maskL  uint64 // Large mask for normalization
}

// NewFastCDC creates a new FastCDC chunker.
func NewFastCDC(data []byte, config Config) *FastCDC {
	return &FastCDC{
		data:   data,
		config: config,
		maskS:  (1 << 15) - 1,
		maskL:  (1 << 11) - 1,
	}
}

// Next returns the next chunk of data.
func (c *FastCDC) Next() (*Chunk, error) {
	if c.pos >= len(c.data) {
		return nil, nil
	}

	remaining := len(c.data) - c.pos
	if remaining <= c.config.MinSize {
		chunk := &Chunk{
			Offset: c.pos,
			Length: remaining,
			Hash:   0,
		}
		c.pos = len(c.data)
		return chunk, nil
	}

	// Placeholder for FastCDC core logic
	length := c.config.AvgSize
	if length > remaining {
		length = remaining
	}

	chunk := &Chunk{
		Offset: c.pos,
		Length: length,
		Hash:   0,
	}

	c.pos += length
	return chunk, nil
}
