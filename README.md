# BitTorrent-rs

Rust로 구현한 BitTorrent 클라이언트입니다.
BEP 3 (BitTorrent Protocol Specification)을 기반으로 제작되었습니다.

## 프로젝트 구조

```
src/
├── main.rs           # 진입점
├── error.rs          # 에러 타입 정의
├── bencode/          # Bencode 인코딩/디코딩
│   ├── mod.rs
│   ├── value.rs      # BencodeValue 타입
│   ├── encoder.rs    # 인코더
│   └── decoder.rs    # 디코더
├── torrent/          # .torrent 파일 파싱
│   ├── mod.rs
│   ├── metainfo.rs   # Metainfo 구조체
│   └── piece.rs      # Piece 해시 관리
├── tracker/          # Tracker 통신
│   ├── mod.rs
│   ├── client.rs     # Tracker 클라이언트
│   ├── peer.rs       # Peer 정보
│   ├── request.rs    # Tracker 요청
│   └── response.rs   # Tracker 응답
├── peer/             # Peer 프로토콜
│   ├── mod.rs
│   ├── connection.rs # Peer 연결 관리
│   ├── message.rs    # Peer 메시지 타입
│   └── protocol.rs   # Handshake 프로토콜
├── piece/            # Piece 관리
│   ├── mod.rs
│   ├── manager.rs    # Piece 다운로드 관리
│   └── picker.rs     # Piece 선택 전략 (Rarest-first)
├── storage/          # 파일 I/O
│   └── mod.rs        # StorageManager
├── client/           # 클라이언트 오케스트레이터
│   └── mod.rs        # TorrentClient
└── cli/              # CLI 인터페이스
    └── mod.rs
```

## 주요 컴포넌트

### 1. Bencode (완료 ✅)
- BitTorrent에서 사용하는 인코딩 형식
- Integer, String, List, Dictionary 지원
- 인코딩/디코딩 완전 구현

### 2. Torrent 메타정보 파서 (완료 ✅)
- `.torrent` 파일 파싱
- Info hash 계산
- 단일/멀티 파일 모드 지원

### 3. Tracker 클라이언트 (완료 ✅)
- HTTP tracker 통신
- Peer 리스트 조회
- Compact/Dictionary 형식 지원

### 4. Peer 프로토콜 (기본 구조 완료 🔨)
- Handshake 프로토콜
- Peer 메시지 직렬화/역직렬화
- TCP 연결 관리
- **TODO**: 실제 다운로드 로직 구현 필요

### 5. Piece 관리 (기본 구조 완료 🔨)
- Rarest-first 전략
- Piece 검증 (SHA1)
- Block 단위 다운로드
- **TODO**: Endgame 모드 구현 필요

### 6. Storage 관리 (기본 구조 완료 🔨)
- 멀티 파일 지원
- Global offset 기반 I/O
- **TODO**: Resume 기능 구현 필요

### 7. Client 오케스트레이터 (기본 구조 완료 🔨)
- 모든 컴포넌트 조율
- **TODO**: 전체 다운로드 플로우 구현 필요

## 빌드 및 실행

```bash
# 빌드
cargo build

# 테스트
cargo test

# 실행
cargo run -- --help

# Torrent 정보 보기
cargo run -- info <torrent-file>

# Torrent 다운로드
cargo run -- download -t <torrent-file> -o <output-dir>
```

## 현재 상태

### 완료된 기능
- ✅ Bencode 인코딩/디코딩
- ✅ .torrent 파일 파싱
- ✅ Info hash 계산
- ✅ Tracker 통신 및 peer 리스트 조회
- ✅ Peer 프로토콜 메시지 정의
- ✅ Piece 관리 기본 구조
- ✅ 파일 I/O 기본 구조
- ✅ CLI 인터페이스

### 구현 필요 사항

#### 1. Peer 다운로드 로직
현재 `client/mod.rs`의 `download` 함수는 tracker에서 peer 리스트만 가져오고 실제 다운로드는 하지 않습니다.

**구현이 필요한 부분:**
- [ ] Peer 연결 풀 관리
- [ ] 비동기 다운로드 (여러 peer에서 동시에)
- [ ] Choking 알고리즘 (Tit-for-tat)
- [ ] Request pipelining (한 번에 여러 block 요청)

#### 2. Piece 선택 최적화
- [ ] Endgame 모드 (마지막 몇 개 piece 처리)
- [ ] Random first piece (초기 랜덤 선택)

#### 3. Resume 기능
- [ ] 다운로드 상태 저장
- [ ] 이미 다운로드된 piece 검증 및 재개

#### 4. DHT (분산 해시 테이블)
- [ ] Trackerless 토렌트 지원
- [ ] BEP 5 구현

#### 5. 성능 최적화
- [ ] Disk I/O 버퍼링
- [ ] 메모리 풀 사용
- [ ] Zero-copy 최적화

#### 6. 추가 기능
- [ ] Seeding (업로드)
- [ ] UPnP/NAT-PMP 지원
- [ ] Magnet link 지원
- [ ] WebUI 또는 GUI

## 다음 단계

### 1단계: 단순 다운로드 구현
가장 먼저 구현해야 할 것은 단일 peer로부터 순차적으로 piece를 다운로드하는 기본 로직입니다.

**구현 위치**: `src/client/mod.rs`의 `download` 함수

**구현 내용**:
1. Peer 리스트에서 첫 번째 peer 선택
2. Handshake 및 연결
3. Interested 메시지 전송
4. Unchoke 대기
5. 각 piece에 대해:
   - 필요한 block들 요청
   - Block 수신 및 조합
   - SHA1 검증
   - 디스크에 저장
6. 진행률 표시

### 2단계: 멀티 Peer 다운로드
여러 peer로부터 동시에 다운로드하도록 확장합니다.

### 3단계: 최적화 및 고급 기능
Choking 알고리즘, Endgame 모드, Resume 등을 추가합니다.

## 아키텍처 설계 원칙

이 프로젝트는 확장성을 고려하여 설계되었습니다:

1. **모듈화**: 각 컴포넌트는 독립적으로 테스트 가능
2. **비동기**: Tokio 기반 비동기 I/O
3. **Trait 추상화**: 다양한 구현체 교체 가능
4. **에러 처리**: thiserror 기반 명확한 에러 타입
5. **로깅**: tracing 기반 구조화된 로깅

## 참고 자료

- [BEP 3: The BitTorrent Protocol Specification](https://www.bittorrent.org/beps/bep_0003.html)
- [BitTorrent Protocol](https://wiki.theory.org/BitTorrentSpecification)

## 라이선스

MIT
