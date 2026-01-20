// Package arx provides a public API for the arx journal system.
package arx

import (
	"time"
)

// EntryType represents the type of journal entry.
type EntryType string

// Entry type constants.
const (
	TypeDecision      EntryType = "decision"
	TypeAssumption    EntryType = "assumption"
	TypeClarification EntryType = "clarification"
	TypeBlocker       EntryType = "blocker"
	TypeRisk          EntryType = "risk"
	TypeOverride      EntryType = "override"
	TypeDefer         EntryType = "defer"
	TypeTombstone     EntryType = "tombstone"
)

// EntryState represents the derived state of an entry.
type EntryState string

// Entry state constants.
const (
	StateActive     EntryState = "active"
	StateSuperseded EntryState = "superseded"
	StateReversed   EntryState = "reversed"
)

// Entry represents a journal entry.
type Entry struct {
	ID         string     `json:"id"`
	Type       EntryType  `json:"type"`
	Actor      string     `json:"actor"`
	Date       time.Time  `json:"date"`
	Title      string     `json:"title"`
	Scope      string     `json:"scope,omitempty"`
	Body       string     `json:"body,omitempty"`
	Supersedes string     `json:"supersedes,omitempty"`
	Reverses   string     `json:"reverses,omitempty"`
	State      EntryState `json:"state"`
}
