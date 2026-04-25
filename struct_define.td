fn video_stitching(clips: Vec<Vec<int>>, time: int) -> int {
        let maxn = Vec::<int>::new();

        let i = 0;
        while i < time {
            maxn.push(0);
            i = i + 1;
        }

        let last = 0;
        let pre = 0;

        i = 0;
        let clips_len = clips.len();
        while i < clips_len {
            let clip = clips[i];

            let l = clip[0];
            let r = clip[1];

            if l < time && r > maxn[l] {
                maxn[l] = r;
            }
            i = i + 1;
        }

        let ans = 0;

        i = 0;
        while i < time {
            let v = maxn[i];
            if v > last {
                last = v;
            }

            if i == last {
                return -1;
            }

            if i == pre {
                ans = ans + 1;
                pre = last;
            }

            i = i + 1;
        }

        return ans;
    }

fn push_clip(clips: Vec<Vec<int>>, l: int, r: int) {
    let clip = Vec::<int>::new();
    clip.push(l);
    clip.push(r);
    clips.push(clip);
}

fn main() {
    let clips = Vec::<Vec<int>>::new();

    push_clip(clips, 0, 2);
    push_clip(clips, 4, 6);
    push_clip(clips, 8, 10);
    push_clip(clips, 1, 9);
    push_clip(clips, 1, 5);
    push_clip(clips, 5, 9);
    print(video_stitching(clips, 10));
    return;
}
