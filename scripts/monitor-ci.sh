#!/usr/bin/env bash
# monitor-ci.sh - Monitor GitHub Actions release workflow
#
# Polls the CI workflow triggered by a tag push until all jobs complete
# or a timeout is reached. Reports progress and handles failures.
#
# Usage:
#   bash scripts/monitor-ci.sh <tag>
#
# Arguments:
#   tag  - The git tag that triggered the release workflow (e.g. 1.3.2)
#
# Exit codes:
#   0 - All jobs succeeded
#   1 - One or more jobs failed (failed log summary printed to stderr)
#   2 - Timeout reached
#   3 - Usage error or workflow not found

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/ui.sh"
init_ui

readonly POLL_INTERVAL=30
readonly TIMEOUT=900  # 15 minutes
readonly MAX_WAIT_FOR_RUN=120  # 2 minutes to wait for workflow to appear

# ============================================================================
# Helpers
# ============================================================================

send_notification() {
    local title="$1"
    local message="$2"
    osascript -e "display notification \"$message\" with title \"$title\"" 2>/dev/null || true
}

get_run_id() {
    local tag="$1"
    gh run list --branch "$tag" --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId // empty' 2>/dev/null
}

get_run_status() {
    local run_id="$1"
    gh run view "$run_id" --json status,conclusion,jobs \
        --jq '{status: .status, conclusion: .conclusion, jobs: [.jobs[] | {name: .name, status: .status, conclusion: .conclusion}]}'
}

print_job_progress() {
    local jobs_json="$1"
    local completed failed in_progress queued
    completed=$(echo "$jobs_json" | jq '[.jobs[] | select(.status == "completed")] | length')
    failed=$(echo "$jobs_json" | jq '[.jobs[] | select(.conclusion == "failure")] | length')
    in_progress=$(echo "$jobs_json" | jq '[.jobs[] | select(.status == "in_progress")] | length')
    queued=$(echo "$jobs_json" | jq '[.jobs[] | select(.status == "queued" or .status == "waiting")] | length')
    local total
    total=$(echo "$jobs_json" | jq '.jobs | length')

    local msg="进度: ${completed}/${total} 完成"
    [[ "$in_progress" -gt 0 ]] && msg+=", ${in_progress} 运行中"
    [[ "$queued" -gt 0 ]] && msg+=", ${queued} 排队中"
    [[ "$failed" -gt 0 ]] && msg+=", ${failed} 失败"
    print_info "$msg"
}

# ============================================================================
# Main
# ============================================================================

main() {
    if [[ $# -eq 0 ]]; then
        print_error "Usage: bash scripts/monitor-ci.sh <tag>"
        exit 3
    fi

    local tag="$1"
    print_header "监控 CI 构建: $tag"
    print_separator

    # Wait for workflow run to appear
    local run_id=""
    local waited=0
    while [[ -z "$run_id" && "$waited" -lt "$MAX_WAIT_FOR_RUN" ]]; do
        run_id=$(get_run_id "$tag")
        if [[ -z "$run_id" ]]; then
            print_info "等待 workflow 启动... (${waited}s)"
            sleep 10
            waited=$((waited + 10))
        fi
    done

    if [[ -z "$run_id" ]]; then
        print_error "未找到 tag $tag 触发的 workflow run"
        exit 3
    fi

    local repo_url
    repo_url=$(gh repo view --json url --jq '.url' 2>/dev/null || echo "")
    print_info "Workflow Run: ${repo_url}/actions/runs/${run_id}"
    echo ""

    # Poll until completion or timeout
    local elapsed=0
    while [[ "$elapsed" -lt "$TIMEOUT" ]]; do
        local status_json
        status_json=$(get_run_status "$run_id")

        local status conclusion
        status=$(echo "$status_json" | jq -r '.status')
        conclusion=$(echo "$status_json" | jq -r '.conclusion // "null"')

        print_job_progress "$status_json"

        if [[ "$status" == "completed" ]]; then
            echo ""
            if [[ "$conclusion" == "success" ]]; then
                print_success "所有 job 构建成功"
                send_notification "CI 构建成功" "$tag 所有 job 构建成功"
                echo ""
                print_info "Release: ${repo_url}/releases/tag/${tag}"
                exit 0
            else
                print_error "构建失败 (conclusion: $conclusion)"
                send_notification "CI 构建失败" "$tag 构建失败"
                echo ""
                # Print failed jobs
                local failed_jobs
                failed_jobs=$(echo "$status_json" | jq -r '.jobs[] | select(.conclusion == "failure") | .name')
                if [[ -n "$failed_jobs" ]]; then
                    print_info "失败的 job:"
                    echo "$failed_jobs" | while read -r job; do
                        echo "  - $job"
                    done
                fi
                echo ""
                print_info "失败日志:"
                gh run view "$run_id" --log-failed 2>&1 | tail -50 || true
                exit 1
            fi
        fi

        sleep "$POLL_INTERVAL"
        elapsed=$((elapsed + POLL_INTERVAL))
    done

    print_error "监控超时 (${TIMEOUT}s)"
    send_notification "CI 监控超时" "$tag 构建仍在运行"
    print_info "构建仍在运行，请手动检查: ${repo_url}/actions/runs/${run_id}"
    exit 2
}

main "$@"
