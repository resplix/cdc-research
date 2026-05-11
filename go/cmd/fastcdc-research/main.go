package main

import (
	"fmt"
	"strings"

	"github.com/Resplix/cdc-research/go/pkg/fastcdc"
)

func main() {
	config := fastcdc.DefaultConfig()
	fmt.Println("Resplix CDC Research Cluster - Go Implementation")
	fmt.Printf("Config: %+v\n", config)

	// Create a less repetitive string for testing
	var sb strings.Builder
	for i := 0; i < 5000; i++ {
		sb.WriteString(fmt.Sprintf("Line %d: Random data %d to ensure chunks are unique... ", i, (i*37)%100))
	}
	original := sb.String()

	testInsertion(original, "INSERTED_DATA_HERE_TO_SHIFT_BYTES", config)
}

func testInsertion(original, insertion string, config fastcdc.Config) {
	data1 := []byte(original)
	
	// Create data2 by inserting into data1
	pos := len(data1) / 4
	data2 := make([]byte, 0, len(data1)+len(insertion))
	data2 = append(data2, data1[:pos]...)
	data2 = append(data2, []byte(insertion)...)
	data2 = append(data2, data1[pos:]...)

	fmt.Printf("Testing insertion of '%s' at position %d\n", insertion, pos)
	compareDedupe(data1, data2, config)
}

func compareDedupe(data1, data2 []byte, config fastcdc.Config) {
	cdc1 := fastcdc.NewFastCDC(data1, config)
	hashes1 := make(map[[32]byte]bool)
	count1 := 0
	for {
		chunk, err := cdc1.Next()
		if err != nil {
			panic(err)
		}
		if chunk == nil {
			break
		}
		hashes1[chunk.ContentHash] = true
		count1++
	}

	cdc2 := fastcdc.NewFastCDC(data2, config)
	count2 := 0
	duplicates := 0
	for {
		chunk, err := cdc2.Next()
		if err != nil {
			panic(err)
		}
		if chunk == nil {
			break
		}
		if hashes1[chunk.ContentHash] {
			duplicates++
		}
		count2++
	}

	fmt.Println("--- Deduplication Report ---")
	fmt.Printf("File 1 Chunks: %d\n", count1)
	fmt.Printf("File 2 Chunks: %d\n", count2)
	fmt.Printf("Duplicate Chunks: %d\n", duplicates)
	if count2 > 0 {
		fmt.Printf("Dedupe Ratio: %.2f%%\n", (float64(duplicates)/float64(count2))*100.0)
	}
}
