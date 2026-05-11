package fastcdc

import (
	"io"

	"lukechampine.com/blake3"
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
	Offset      int
	Length      int
	RollingHash uint64
	ContentHash [32]byte
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
	len    int
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
		maskS:  0x0003590703530000,
		maskL:  0x0000d90003530000,
	}
}

func (s *StreamingChunker) fillBuffer() error {
	if s.pos > 0 {
		copy(s.buf, s.buf[s.pos:s.len])
		s.len -= s.pos
		s.pos = 0
	}

	n, err := s.reader.Read(s.buf[s.len:])
	if n > 0 {
		s.len += n
	}
	if err == io.EOF {
		s.eof = true
		return nil
	}
	return err
}

func (s *StreamingChunker) Next() (*Chunk, error) {
	if s.eof && s.pos >= s.len {
		return nil, nil
	}

	if !s.eof && (s.len-s.pos) < s.config.MaxSize {
		if err := s.fillBuffer(); err != nil {
			return nil, err
		}
	}

	remaining := s.len - s.pos
	if remaining == 0 {
		return nil, nil
	}

	start := s.pos
	var end int
	var hash uint64

	if remaining <= s.config.MinSize {
		end = s.len
	} else {
		end = start + s.config.MinSize
		max := start + s.config.MaxSize
		if max > s.len {
			max = s.len
		}
		avg := start + s.config.AvgSize

		limitS := avg
		if limitS > max {
			limitS = max
		}

		found := false
		for end < limitS {
			hash = (hash << 1) + GearTable[s.buf[end]]
			if (hash & s.maskS) == 0 {
				end = end + 1
				found = true
				break
			}
			end++
		}

		if !found {
			for end < max {
				hash = (hash << 1) + GearTable[s.buf[end]]
				if (hash & s.maskL) == 0 {
					end = end + 1
					found = true
					break
				}
				end++
			}
		}

		if !found {
			end = max
		}
	}

	length := end - start
	contentHash := blake3.Sum256(s.buf[start:end])

	s.pos = end
	return &Chunk{
		Offset:      start,
		Length:      length,
		RollingHash: hash,
		ContentHash: contentHash,
	}, nil
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
		maskS:  0x0003590703530000,
		maskL:  0x0000d90003530000,
	}
}

// Next returns the next chunk of data.
func (c *FastCDC) Next() (*Chunk, error) {
	if c.pos >= len(c.data) {
		return nil, nil
	}

	start := c.pos
	remaining := len(c.data) - start
	var end int
	var hash uint64

	if remaining <= c.config.MinSize {
		end = len(c.data)
	} else {
		end = start + c.config.MinSize
		max := start + c.config.MaxSize
		if max > len(c.data) {
			max = len(c.data)
		}
		avg := start + c.config.AvgSize

		limitS := avg
		if limitS > max {
			limitS = max
		}

		found := false
		for end < limitS {
			hash = (hash << 1) + GearTable[c.data[end]]
			if (hash & c.maskS) == 0 {
				end = end + 1
				found = true
				break
			}
			end++
		}

		if !found {
			for end < max {
				hash = (hash << 1) + GearTable[c.data[end]]
				if (hash & c.maskL) == 0 {
					end = end + 1
					found = true
					break
				}
				end++
			}
		}

		if !found {
			end = max
		}
	}

	length := end - start
	contentHash := blake3.Sum256(c.data[start:end])

	c.pos = end
	return &Chunk{
		Offset:      start,
		Length:      length,
		RollingHash: hash,
		ContentHash: contentHash,
	}, nil
}
