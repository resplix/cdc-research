package fastcdc


import (
	"io"
)

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

// StreamingChunker handles CDC on an io.Reader stream.
type StreamingChunker struct {
	reader io.Reader
	buf    []byte
	pos    int
	config Config
	maskS  uint64
	maskL  uint64
	eof    bool
}

// NewStreamingChunker creates a new Chunker for an io.Reader.
func NewStreamingChunker(r io.Reader, config Config) *StreamingChunker {
	return &StreamingChunker{
		reader: r,
		buf:    make([]byte, config.MaxSize*2),
		config: config,
		maskS:  (1 << 15) - 1,
		maskL:  (1 << 11) - 1,
	}
}

func (s *StreamingChunker) Next() (*Chunk, error) {
	// Implementation for streaming would involve managing the buffer
	// and reading from the reader when needed.
	// For now, this is a placeholder to show architectural intent.
	return nil, io.EOF
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

	hash := uint64(0)
	start := c.pos
	end := start + c.config.MinSize
	max := start + c.config.MaxSize
	if max > len(c.data) {
		max = len(c.data)
	}
	avg := start + c.config.AvgSize

	// Phase 1: Normalized Chunking with small mask
	limitS := avg
	if limitS > max {
		limitS = max
	}
	for end < limitS {
		hash = (hash << 1) + GearTable[c.data[end]]
		if (hash & c.maskS) == 0 {
			length := (end + 1) - start
			c.pos = end + 1
			return &Chunk{Offset: start, Length: length, Hash: hash}, nil
		}
		end++
	}

	// Phase 2: Normalized Chunking with large mask
	for end < max {
		hash = (hash << 1) + GearTable[c.data[end]]
		if (hash & c.maskL) == 0 {
			length := (end + 1) - start
			c.pos = end + 1
			return &Chunk{Offset: start, Length: length, Hash: hash}, nil
		}
		end++
	}

	// Phase 3: Max size reached
	length := max - start
	c.pos = max
	return &Chunk{
		Offset: start,
		Length: length,
		Hash:   hash,
	}, nil
}
