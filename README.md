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

### 4. Peer 프로토콜 (완료 ✅)
- Handshake 프로토콜
- Peer 메시지 직렬화/역직렬화
- TCP 연결 관리
- 다운로드 로직 구현
- 에러 처리 및 타임아웃
- Peer 상태 관리

### 5. Piece 관리 (완료 ✅)
- Rarest-first 전략
- Random first piece 전략
- Piece 검증 (SHA1)
- Block 단위 다운로드
- Endgame 모드 구현

### 6. Storage 관리 (기본 구조 완료 🔨)
- 멀티 파일 지원
- Global offset 기반 I/O
- **TODO**: Resume 기능 구현 필요

### 7. Client 오케스트레이터 (완료 ✅)
- 모든 컴포넌트 조율
- 다중 peer 동시 다운로드
- 진행률 모니터링
- 자동 재시도 로직

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
- ✅ Peer 프로토콜 메시지 정의 및 통신
- ✅ Piece 관리 (다운로드, 검증, 저장)
- ✅ 파일 I/O 기본 구조
- ✅ CLI 인터페이스
- ✅ 다중 peer 동시 다운로드
- ✅ Random first piece 전략
- ✅ Rarest-first piece 선택
- ✅ Endgame 모드
- ✅ 진행률 모니터링
- ✅ 에러 처리 및 타임아웃

### 구현 필요 사항

#### 1. 고급 Peer 관리
- [ ] Choking 알고리즘 (Tit-for-tat)
- [ ] Request pipelining (한 번에 여러 block 요청)
- [ ] Peer 연결 풀 최적화

#### 2. Resume 기능
- [ ] 다운로드 상태 저장
- [ ] 이미 다운로드된 piece 검증 및 재개

#### 3. DHT (분산 해시 테이블)
- [ ] Trackerless 토렌트 지원
- [ ] BEP 5 구현

#### 4. 성능 최적화
- [ ] Disk I/O 버퍼링
- [ ] 메모리 풀 사용
- [ ] Zero-copy 최적화

#### 5. 추가 기능
- [ ] Seeding (업로드)
- [ ] UPnP/NAT-PMP 지원
- [ ] Magnet link 지원
- [ ] WebUI 또는 GUI

## 다음 단계

### ~~1단계: 단순 다운로드 구현~~ ✅ 완료
~~가장 먼저 구현해야 할 것은 단일 peer로부터 순차적으로 piece를 다운로드하는 기본 로직입니다.~~

**구현 완료**:
1. ✅ Peer 리스트에서 peer 선택 및 연결
2. ✅ Handshake 및 연결 관리
3. ✅ Interested/Unchoke 메시지 처리
4. ✅ Block 요청 및 수신
5. ✅ Piece 검증 및 저장
6. ✅ 진행률 표시

### ~~2단계: 멀티 Peer 다운로드~~ ✅ 완료
~~여러 peer로부터 동시에 다운로드하도록 확장합니다.~~

**구현 완료**:
1. ✅ 다중 peer 연결
2. ✅ 동시 다운로드 작업 관리
3. ✅ Piece picker를 통한 작업 분배
4. ✅ Random first & Rarest-first 전략
5. ✅ Endgame 모드

### 3단계: 최적화 및 고급 기능
다음으로 구현할 기능들:
1. Resume 기능 (다운로드 재개)
2. Choking 알고리즘 최적화
3. Request pipelining
4. DHT 지원 (Trackerless torrents)

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
