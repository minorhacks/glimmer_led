package main


import (
	"flag"
	"log"
)

func main() {
	targetIP := flag.String("target_ip", "127.0.0.1", "Target UDP IP address")
	targetPort := flag.Int("target_port", 9999, "Target UDP port")
	httpPort := flag.Int("http_port", 8080, "HTTP server port")
	numPixels := flag.Int("num_pixels", 50, "Number of pixels on the strip")
	flag.Parse()

	server, err := NewServer(*targetIP, *targetPort, *numPixels)
	if err != nil {
		log.Fatalf("Failed to create server: %v", err)
	}

	log.Printf("Starting Glimmer Server on port %d, targeting %s:%d", *httpPort, *targetIP, *targetPort)
	if err := server.Start(*httpPort); err != nil {
		log.Fatalf("Server exited: %v", err)
	}
}