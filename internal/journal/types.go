package journal

import (
	"time"
)

// EntryType represents the type of journal entry
type EntryType string

const (
	EntryTypeClarification EntryType = "clarification"
	EntryTypeDecision      EntryType = "decision"
	EntryTypeOverride      EntryType = "override"
	EntryTypeBlocker       EntryType = "blocker"
	EntryTypeAssumption    EntryType = "assumption"
	EntryTypeRisk          EntryType = "risk"
	EntryTypeDefer         EntryType = "defer"
	EntryTypeTombstone     EntryType = "tombstone"
)

// ActorType represents who created the entry
type ActorType string

const (
	ActorHuman    ActorType = "human"
	ActorPlanner  ActorType = "planner"
	ActorExecutor ActorType = "executor"
	ActorReviewer ActorType = "reviewer"
	ActorSystem   ActorType = "system"
)

// EntryState represents the derived state of an entry
type EntryState string

const (
	StateActive     EntryState = "active"
	StateSuperseded EntryState = "superseded"
	StateReversed   EntryState = "reversed"
)

// Entry represents a journal entry
type Entry struct {
	ID          string    `yaml:"id"`
	Type        EntryType `yaml:"type"`
	Actor       ActorType `yaml:"actor"`
	Date        time.Time `yaml:"date"`
	Title       string    `yaml:"title"`
	Scope       string    `yaml:"scope,omitempty"`
	Content     string    `yaml:"-"`
	Supersedes  string    `yaml:"supersedes,omitempty"`
	ReversedBy  string    `yaml:"reversed_by,omitempty"`
}

// DerivedState holds the computed state of all entries
type DerivedState struct {
	Entries map[string]*EntryWithState
}

// EntryWithState is an entry with its computed state
type EntryWithState struct {
	Entry
	State EntryState
}
