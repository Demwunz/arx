package journal

import (
	"bufio"
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"gopkg.in/yaml.v3"
)

const (
	JournalDir = ".arx/journal"
)

// Write saves an entry with YAML frontmatter to the journal directory
func Write(entry *Entry) error {
	if err := os.MkdirAll(JournalDir, 0755); err != nil {
		return fmt.Errorf("failed to create journal directory: %w", err)
	}

	frontmatter, err := yaml.Marshal(entry)
	if err != nil {
		return fmt.Errorf("failed to marshal entry: %w", err)
	}

	var buf bytes.Buffer
	buf.WriteString("---\n")
	buf.Write(frontmatter)
	buf.WriteString("---\n")
	if entry.Content != "" {
		buf.WriteString("\n")
		buf.WriteString(entry.Content)
	}

	filePath := filepath.Join(JournalDir, entry.ID+".md")
	if err := os.WriteFile(filePath, buf.Bytes(), 0644); err != nil {
		return fmt.Errorf("failed to write entry file: %w", err)
	}

	return nil
}

// ReadAll loads all entries from the journal directory sorted by date
func ReadAll() ([]*Entry, error) {
	entries, err := os.ReadDir(JournalDir)
	if err != nil {
		if os.IsNotExist(err) {
			return []*Entry{}, nil
		}
		return nil, fmt.Errorf("failed to read journal directory: %w", err)
	}

	var result []*Entry
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".md") {
			continue
		}

		filePath := filepath.Join(JournalDir, e.Name())
		entry, err := readEntry(filePath)
		if err != nil {
			return nil, fmt.Errorf("failed to read entry %s: %w", e.Name(), err)
		}
		result = append(result, entry)
	}

	sort.Slice(result, func(i, j int) bool {
		return result[i].Date.Before(result[j].Date)
	})

	return result, nil
}

func readEntry(filePath string) (*Entry, error) {
	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, err
	}

	scanner := bufio.NewScanner(bytes.NewReader(data))
	var frontmatterLines []string
	var contentLines []string
	inFrontmatter := false
	frontmatterDone := false

	for scanner.Scan() {
		line := scanner.Text()
		if line == "---" {
			if !inFrontmatter && !frontmatterDone {
				inFrontmatter = true
				continue
			} else if inFrontmatter {
				inFrontmatter = false
				frontmatterDone = true
				continue
			}
		}
		if inFrontmatter {
			frontmatterLines = append(frontmatterLines, line)
		} else if frontmatterDone {
			contentLines = append(contentLines, line)
		}
	}

	entry := &Entry{}
	frontmatterYAML := strings.Join(frontmatterLines, "\n")
	if err := yaml.Unmarshal([]byte(frontmatterYAML), entry); err != nil {
		return nil, fmt.Errorf("failed to unmarshal frontmatter: %w", err)
	}

	content := strings.Join(contentLines, "\n")
	entry.Content = strings.TrimSpace(content)

	return entry, nil
}

// GetState computes the derived state of all entries
func GetState() (*DerivedState, error) {
	entries, err := ReadAll()
	if err != nil {
		return nil, err
	}

	state := &DerivedState{
		Entries: make(map[string]*EntryWithState),
	}

	// First pass: add all entries as active
	for _, e := range entries {
		state.Entries[e.ID] = &EntryWithState{
			Entry: *e,
			State: StateActive,
		}
	}

	// Second pass: mark superseded and reversed entries
	for _, e := range entries {
		if e.Supersedes != "" {
			if superseded, ok := state.Entries[e.Supersedes]; ok {
				superseded.State = StateSuperseded
			}
		}
		if e.ReversedBy != "" {
			if current, ok := state.Entries[e.ID]; ok {
				current.State = StateReversed
			}
		}
	}

	return state, nil
}
