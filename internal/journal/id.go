package journal

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"time"
)

// GenerateID creates a unique ID in format: {type}-{YYYY-MM-DD}-{6-char-hex}
func GenerateID(entryType EntryType) (string, error) {
	bytes := make([]byte, 3)
	if _, err := rand.Read(bytes); err != nil {
		return "", fmt.Errorf("failed to generate random bytes: %w", err)
	}
	
	dateStr := time.Now().Format("2006-01-02")
	hexStr := hex.EncodeToString(bytes)
	
	return fmt.Sprintf("%s-%s-%s", entryType, dateStr, hexStr), nil
}
