package main

import (
	"fmt"
	"net"
	"net/http"
	"sync"
	"time"
)

// Server controls the light strips
type Server struct {
	targetAddr *net.UDPAddr
	conn       *net.UDPConn
	
	mu            sync.Mutex
	activePattern Pattern
	
	// Config
	numPixels int
	fps       int
}

// Pattern generates frame data
type Pattern interface {
	NextFrame(t float64) []byte
}

func NewServer(targetIP string, targetPort int, numPixels int) (*Server, error) {
	addr, err := net.ResolveUDPAddr("udp", fmt.Sprintf("%s:%d", targetIP, targetPort))
	if err != nil {
		return nil, fmt.Errorf("resolving udp addr: %w", err)
	}

	conn, err := net.DialUDP("udp", nil, addr)
	if err != nil {
		return nil, fmt.Errorf("dialing udp: %w", err)
	}

	s := &Server{
		targetAddr: addr,
		conn:       conn,
		numPixels:  numPixels,
		fps:        30, // Default FPS
	}
	s.SetPattern(&SolidColorPattern{R: 0, G: 0, B: 0}) // Start off
	return s, nil
}

func (s *Server) Start(httpPort int) error {
	go s.renderLoop()

	http.HandleFunc("/pattern/red", func(w http.ResponseWriter, r *http.Request) {
		s.SetPattern(&SolidColorPattern{R: 255, G: 0, B: 0})
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("Pattern set to Red\n"))
	})
	
	http.HandleFunc("/pattern/green", func(w http.ResponseWriter, r *http.Request) {
		s.SetPattern(&SolidColorPattern{R: 0, G: 255, B: 0})
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("Pattern set to Green\n"))
	})

	http.HandleFunc("/pattern/blue", func(w http.ResponseWriter, r *http.Request) {
		s.SetPattern(&SolidColorPattern{R: 0, G: 0, B: 255})
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("Pattern set to Blue\n"))
	})
	
	http.HandleFunc("/off", func(w http.ResponseWriter, r *http.Request) {
		s.SetPattern(&SolidColorPattern{R: 0, G: 0, B: 0})
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("Lights off\n"))
	})

	addr := fmt.Sprintf(":%d", httpPort)
	fmt.Printf("Listening on %s...\n", addr)
	return http.ListenAndServe(addr, nil)
}

func (s *Server) SetPattern(p Pattern) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.activePattern = p
}

func (s *Server) renderLoop() {
	ticker := time.NewTicker(time.Second / time.Duration(s.fps))
	startTime := time.Now()

	for range ticker.C {
		s.mu.Lock()
		p := s.activePattern
		params := time.Since(startTime).Seconds()
		frame := p.NextFrame(params)
		
		// If pattern returns single pixel (3 bytes), fill the whole strip
		if len(frame) == 3 {
			fullFrame := make([]byte, s.numPixels*3)
			for i := 0; i < s.numPixels; i++ {
				fullFrame[i*3] = frame[0]
				fullFrame[i*3+1] = frame[1]
				fullFrame[i*3+2] = frame[2]
			}
			frame = fullFrame
		} else if len(frame) != s.numPixels * 3 {
             // Handle resize if needed, or just let it send potentially wrong size (or truncated)
             // For now, assume pattern is correct if not 3 bytes.
             if len(frame) < s.numPixels * 3 {
                 extended := make([]byte, s.numPixels * 3)
                 copy(extended, frame)
                 frame = extended
             }
        }
		
		s.mu.Unlock()

		_, err := s.conn.Write(frame)
		if err != nil {
			fmt.Printf("Error sending UDP: %v\n", err)
		}
	}
}

// -- Patterns --

type SolidColorPattern struct {
	R, G, B byte
}

func (p *SolidColorPattern) NextFrame(t float64) []byte {
    return []byte{p.R, p.G, p.B}
}
