# Premiere Pro Development Agent Instructions

## Primary Mission
Complete the Premiere Pro UXP plugin implementation with a comprehensive command set, focusing on performance and stability.

## Key Implementation Areas

### 1. Timeline Manipulation
- [ ] Implement sequence creation and management
- [ ] Add clip insertion and arrangement
- [ ] Create transition application system
- [ ] Build track management (video/audio)
- [ ] Implement marker system

### 2. Effect System
- [ ] Create effect application framework
- [ ] Implement effect parameter control
- [ ] Add preset management
- [ ] Build effect animation system
- [ ] Create custom effect chains

### 3. Export/Render Operations
- [ ] Implement export preset selection
- [ ] Create render queue management
- [ ] Add progress monitoring
- [ ] Build batch export system
- [ ] Implement format-specific options

### 4. Project Management
- [ ] Create project file operations
- [ ] Implement bin organization
- [ ] Add metadata management
- [ ] Build search functionality
- [ ] Create project templates

### 5. Media Management
- [ ] Implement media import system
- [ ] Create proxy workflow
- [ ] Add media replacement
- [ ] Build conform tools
- [ ] Implement cache management

## Performance Requirements
- Command execution < 100ms for basic operations
- Batch operations with progress feedback
- Efficient memory usage
- Proper error handling and recovery

## Testing Requirements
- Test with 4K and 8K footage
- Verify with complex timelines (100+ clips)
- Test all export formats
- Ensure stability over long sessions