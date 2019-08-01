let data = [
    0, 2, 5, 1,
    0, 3, 1, 7,
    0, 10, 5, 1,
    4, 0, 14, 1,
    4, 1, 1, 1,
    4, 11, 1, 1,
    4, 12, 14, 1,
    17, 1, 1, 1,
    17, 2, 5, 1,
    17, 10, 5, 1,
    17, 11, 1, 1,
    21, 3, 1, 7,
    3, 4, 2, 1,
    3, 6, 1, 1,
    3, 8, 2, 1,
    6, 2, 4, 1,
    6, 10, 2, 1,
    7, 4, 1, 2,
    7, 7, 1, 2,
    9, 4, 4, 1,
    9, 8, 4, 1,
    9, 9, 1, 2,
    12, 2, 1, 2,
    12, 10, 4, 1,
    14, 2, 2, 1,
    14, 4, 1, 2,
    14, 7, 1, 2,
    17, 4, 2, 1,
    17, 8, 2, 1,
    18, 6, 1, 1
];
let result = "";
for (i = 0; i < data.length; i += 4) {
    let x = data[i];
    let y = data[i + 1];
    let w = data[i + 2];
    let h = data[i + 3];
    result += "(data: Some((" +
        "transform: Some((translation: (" + String(x * 32 + w * 16) + ", " + String(y * 32 + h * 16) + ", 0),rotation: (1, 0, 0, 0),scale: (" + String(w) + ", " + String(h) + ", 1),))," +
        "collider: Some((tag: \"Wall\",width: " + String(w * 32) + ",height: " + String(h * 32) + ",))," +
        "sprite: Some((sprite_number: 0,))," +
        ")),),\n";
}
console.log(result);
