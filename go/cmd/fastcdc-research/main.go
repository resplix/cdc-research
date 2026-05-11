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

	// Experiment 1: Byte-Shift Resilience
	fmt.Println("\n[EXPERIMENT 1: Byte-Shift Resilience]")
	base, _ := os.ReadFile("../tests/data/random_base.txt")
	shifted, _ := os.ReadFile("../tests/data/random_shifted.txt")

	fmt.Println("Comparing 'random_base.txt' vs 'random_shifted.txt' (1-byte shift at start)")
	compareDedupe(base, shifted, config)

	// Experiment 2: Modification Resilience
	fmt.Println("\n[EXPERIMENT 2: Tail Modification Resilience]")
	baseLarge, _ := os.ReadFile("../tests/data/large_base.txt")
	modifiedLarge, _ := os.ReadFile("../tests/data/large_modified.txt")

	fmt.Println("Comparing 'large_base.txt' vs 'large_modified.txt' (appended content)")
	compareDedupe(baseLarge, modifiedLarge, config)
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
