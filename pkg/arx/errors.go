package arx

import "errors"

// Exported errors for the arx package.
var (
	// ErrNotFound is returned when an entry is not found.
	ErrNotFound = errors.New("entry not found")

	// ErrInvalidType is returned when an invalid entry type is provided.
	ErrInvalidType = errors.New("invalid entry type")
)
