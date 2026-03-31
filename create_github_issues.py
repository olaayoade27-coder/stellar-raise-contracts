#!/usr/bin/env python3

import re
import subprocess
import time
import sys

def extract_issues_from_file(filename):
    """Extract issues from the issue.md file"""
    with open(filename, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Split by --- to get individual issues
    sections = content.split('---')
    
    issues = []
    for section in sections:
        section = section.strip()
        if not section:
            continue
            
        # Check if this section starts with a numbered issue
        match = re.match(r'^(\d+)\. Great issue: (.+)', section, re.MULTILINE)
        if match:
            issue_num = match.group(1)
            title = match.group(2)
            
            # The rest is the body
            body_start = section.find('\nDescription')
            if body_start != -1:
                body = section[body_start:].strip()
            else:
                body = section
            
            issues.append({
                'number': issue_num,
                'title': f"{issue_num}. Great issue: {title}",
                'body': body
            })
    
    return issues

def create_github_issue(title, body, repo):
    """Create a GitHub issue using gh CLI"""
    cmd = [
        'gh', 'issue', 'create',
        '--title', title,
        '--body', body,
        '--repo', repo
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Error creating issue '{title}': {e.stderr}")
        return None

def main():
    filename = 'issue.md'
    repo = 'Crowdfunding-DApp/stellar-raise-contracts'
    
    print("Extracting issues from issue.md...")
    issues = extract_issues_from_file(filename)
    
    print(f"Found {len(issues)} issues")
    
    # Ask for confirmation
    response = input(f"Do you want to create {len(issues)} issues on GitHub? (y/N): ")
    if response.lower() != 'y':
        print("Aborted.")
        return
    
    success_count = 0
    error_count = 0
    
    for i, issue in enumerate(issues, 1):
        print(f"Creating issue {i}/{len(issues)}: {issue['title'][:50]}...")
        
        url = create_github_issue(issue['title'], issue['body'], repo)
        
        if url:
            print(f"  ✓ Created: {url}")
            success_count += 1
        else:
            print(f"  ✗ Failed to create issue {issue['number']}")
            error_count += 1
        
        # Rate limiting - wait a bit between issues
        if i < len(issues):
            time.sleep(2)
    
    print(f"\nDone! Created {success_count} issues successfully, {error_count} failed.")

if __name__ == '__main__':
    main()
