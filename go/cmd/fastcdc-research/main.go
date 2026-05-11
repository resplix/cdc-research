package main

import (
	"fmt"
	"os"

	"github.com/Resplix/cdc-research/go/pkg/fastcdc"
)

func main() {
	config := fastcdc.DefaultConfig()
	fmt.Println("====================================================")
	fmt.Println("/// RESPLIX CDC EXPERIMENTAL RESEARCH RUNNER (GO)")
	fmt.Println("====================================================")
	fmt.Printf("Config: %+v\n", config)

	baseRand, _ := os.ReadFile("../tests/data/random_base.txt")
	shiftedRand, _ := os.ReadFile("../tests/data/random_shifted.txt")

	// Experiment 1: Byte-Shift Resilience
	fmt.Println("\n[EXPERIMENT 1: Byte-Shift Resilience]")
	compareDedupe(baseRand, shiftedRand, config)

	// Experiment 2: Corruption Resilience
	fmt.Println("\n[EXPERIMENT 2: Single-Byte Corruption]")
	testCorruption(baseRand, config)

	// Experiment 3: Block Reordering
	fmt.Println("\n[EXPERIMENT 3: Block Reordering]")
	testReordering(baseRand, config)

	// Experiment 4: Statistical Distribution
	fmt.Println("\n[EXPERIMENT 4: Statistical Distribution Analysis]")
	cdc := fastcdc.NewFastCDC(baseRand, config)
	var chunks []*fastcdc.Chunk
	for {
		c, _ := cdc.Next()
		if c == nil { break }
		chunks = append(chunks, c)
	}
	printStats(chunks)
}

func printStats(chunks []*fastcdc.Chunk) {
	if len(chunks) == 0 { return }
	min := 1024 * 1024 * 1024
	max := 0
	sum := 0
	for _, c := range chunks {
		if c.Length < min { min = c.Length }
		if c.Length > max { max = c.Length }
		sum += c.Length
	}
	avg := sum / len(chunks)
	fmt.Println("--- Distribution Statistics ---")
	fmt.Printf("Total Chunks: %d\n", len(chunks))
	fmt.Printf("Min Size:     %.2f KB\n", float64(min)/1024.0)
	fmt.Printf("Max Size:     %.2f KB\n", float64(max)/1024.0)
	fmt.Printf("Avg Size:     %.2f KB\n", float64(avg)/1024.0)
	fmt.Printf("Total Data:   %.2f KB\n", float64(sum)/1024.0)
}

func getDuplicates(chunks1, chunks2 []*fastcdc.Chunk) int {
	hashes1 := make(map[[32]byte]bool)
	for _, c := range chunks1 { hashes1[c.ContentHash] = true }
	duplicates := 0
	for _, c := range chunks2 {
		if hashes1[c.ContentHash] { duplicates++ }
	}
	return duplicates
}

func testCorruption(data []byte, config fastcdc.Config) {
	corrupted := make([]byte, len(data))
	copy(corrupted, data)
	if len(corrupted) > 5000 {
		corrupted[5000] ^= 0xFF
	}

	cdc1 := fastcdc.NewFastCDC(data, config)
	var chunks1 []*fastcdc.Chunk
	for {
		c, _ := cdc1.Next()
		if c == nil { break }
		chunks1 = append(chunks1, c)
	}

	cdc2 := fastcdc.NewFastCDC(corrupted, config)
	var chunks2 []*fastcdc.Chunk
	for {
		c, _ := cdc2.Next()
		if c == nil { break }
		chunks2 = append(chunks2, c)
	}

	dups := getDuplicates(chunks1, chunks2)
	fmt.Println("Original vs Corrupted (1-byte change at index 5000)")
	fmt.Printf("Duplicate Chunks: %d / %d\n", dups, len(chunks2))
	fmt.Printf("Dedupe Ratio:     %.2f%%\n", (float64(dups)/float64(len(chunks2)))*100.0)
}

func testReordering(data []byte, config fastcdc.Config) {
	mid := len(data) / 2
	reordered := append([]byte(nil), data[mid:]...)
	reordered = append(reordered, data[:mid]...)

	cdc1 := fastcdc.NewFastCDC(data, config)
	var chunks1 []*fastcdc.Chunk
	for {
		c, _ := cdc1.Next()
		if c == nil { break }
		chunks1 = append(chunks1, c)
	}

	cdc2 := fastcdc.NewFastCDC(reordered, config)
	var chunks2 []*fastcdc.Chunk
	for {
		c, _ := cdc2.Next()
		if c == nil { break }
		chunks2 = append(chunks2, c)
	}

	dups := getDuplicates(chunks1, chunks2)
	fmt.Println("Original vs Reordered (Swapped halves)")
	fmt.Printf("Duplicate Chunks: %d / %d\n", dups, len(chunks2))
	fmt.Printf("Dedupe Ratio:     %.2f%%\n", (float64(dups)/float64(len(chunks2)))*100.0)
}

func compareDedupe(data1, data2 []byte, config fastcdc.Config) {
	cdc1 := fastcdc.NewFastCDC(data1, config)
	var chunks1 []*fastcdc.Chunk
	hashes1 := make(map[[32]byte]bool)
	for {
		chunk, err := cdc1.Next()
		if err != nil {
			panic(err)
		}
		if chunk == nil {
			break
		}
		chunks1 = append(chunks1, chunk)
		hashes1[chunk.ContentHash] = true
	}

	cdc2 := fastcdc.NewFastCDC(data2, config)
	var chunks2 []*fastcdc.Chunk
	for {
		chunk, err := cdc2.Next()
		if err != nil {
			panic(err)
		}
		if chunk == nil {
			break
		}
		chunks2 = append(chunks2, chunk)
	}

	fmt.Println("--- Chunk Breakdown ---")
	fmt.Print("File 1 Chunks: ")
	for _, c := range chunks1 {
		fmt.Printf("[%dKB] ", c.Length/1024)
	}
	fmt.Print("\nFile 2 Chunks: ")
	for _, c := range chunks2 {
		fmt.Printf("[%dKB] ", c.Length/1024)
	}
	fmt.Println("\n")

	duplicates := 0
	for _, chunk := range chunks2 {
		if hashes1[chunk.ContentHash] {
			duplicates++
		}
	}

	fmt.Println("--- Deduplication Report ---")
	fmt.Printf("Duplicate Chunks: %d / %d\n", duplicates, len(chunks2))
	if len(chunks2) > 0 {
		fmt.Printf("Dedupe Ratio: %.2f%%\n", (float64(duplicates)/float64(len(chunks2)))*100.0)
	}
}
