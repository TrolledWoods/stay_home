#[inline]
pub fn matrix_mul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
	let mut result = [[0.0f32; 3]; 3];
	for v in 0..3 {
		result[v] = matrix_vec_mul(a, b[v]);
	}
	result
}

#[inline]
pub fn matrix_vec_mul(mat: [[f32; 3]; 3], vec: [f32; 3]) -> [f32; 3] {
	let mut result = [0.0f32; 3];
	for j in 0..3 {
		for i in 0..3 {
			result[i] += mat[j][i] * vec[j];
		}
	}
	result
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn matrix_vec_mul_test() {
		assert_eq!(
			matrix_vec_mul(
				[[1.0, 0.0, 0.0],
				 [0.0, 1.0, 0.0],
				 [0.0, 0.0, 1.0]],
				[0.2, 0.4, 0.6]
			),
			[0.2, 0.4, 0.6]
		);

		assert_eq!(
			matrix_vec_mul(
				[[2.0, 0.0, 0.0],
				 [0.0, 1.0, 0.0],
				 [0.0, 1.0, 1.0]],
				[0.2, 0.4, 0.6]
			),
			[0.4, 1.0, 0.6]
		);
	}
}
