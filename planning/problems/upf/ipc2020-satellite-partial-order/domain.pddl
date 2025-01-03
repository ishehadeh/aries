(define (domain p1obs_1sat_1mod-domain)
 (:requirements :strips :typing :hierarchy)
 (:types
    direction instrument satellite mode - object
    calib_direction image_direction - direction
 )
 (:predicates (on_board ?arg0 - instrument ?arg1 - satellite) (supports ?arg0 - instrument ?arg1_0 - mode) (pointing ?arg0_0 - satellite ?arg1_1 - direction) (power_avail ?arg0_0 - satellite) (power_on ?arg0 - instrument) (calibrated ?arg0 - instrument) (have_image ?arg0_1 - image_direction ?arg1_0 - mode) (calibration_target ?arg0 - instrument ?arg1_2 - calib_direction))
 (:task do_observation
  :parameters ( ?do_d - image_direction ?do_m - mode))
 (:task activate_instrument
  :parameters ( ?ai_s - satellite ?ai_i - instrument))
 (:task auto_calibrate
  :parameters ( ?ac_s - satellite ?ac_i - instrument))
 (:method method0
  :parameters ( ?mdoatt_t_d_prev - direction ?mdoatt_t_s - satellite ?mdoatt_ti_d - image_direction ?mdoatt_ti_i - instrument ?mdoatt_ti_m - mode)
  :task (do_observation ?mdoatt_ti_d ?mdoatt_ti_m)
  :ordered-subtasks (and
    (task0 (activate_instrument ?mdoatt_t_s ?mdoatt_ti_i))
    (task1 (turn_to ?mdoatt_t_s ?mdoatt_ti_d ?mdoatt_t_d_prev))
    (task2 (take_image ?mdoatt_t_s ?mdoatt_ti_d ?mdoatt_ti_i ?mdoatt_ti_m))))
 (:method method1
  :parameters ( ?mdott_t_d_prev - direction ?mdott_t_s - satellite ?mdott_ti_d - image_direction ?mdott_ti_i - instrument ?mdott_ti_m - mode)
  :task (do_observation ?mdott_ti_d ?mdott_ti_m)
  :ordered-subtasks (and
    (task0 (turn_to ?mdott_t_s ?mdott_ti_d ?mdott_t_d_prev))
    (task1 (take_image ?mdott_t_s ?mdott_ti_d ?mdott_ti_i ?mdott_ti_m))))
 (:method method2
  :parameters ( ?mdoat_ti_d - image_direction ?mdoat_ti_i - instrument ?mdoat_ti_m - mode ?mdoat_ti_s - satellite)
  :task (do_observation ?mdoat_ti_d ?mdoat_ti_m)
  :ordered-subtasks (and
    (task0 (activate_instrument ?mdoat_ti_s ?mdoat_ti_i))
    (task1 (take_image ?mdoat_ti_s ?mdoat_ti_d ?mdoat_ti_i ?mdoat_ti_m))))
 (:method method3
  :parameters ( ?mdot_ti_d - image_direction ?mdot_ti_i - instrument ?mdot_ti_m - mode ?mdot_ti_s - satellite)
  :task (do_observation ?mdot_ti_d ?mdot_ti_m)
  :ordered-subtasks (and
    (task0 (take_image ?mdot_ti_s ?mdot_ti_d ?mdot_ti_i ?mdot_ti_m))))
 (:method method4
  :parameters ( ?maissa_ac_i - instrument ?maissa_ac_s - satellite ?maissa_sof_i - instrument)
  :task (activate_instrument ?maissa_ac_s ?maissa_ac_i)
  :ordered-subtasks (and
    (task0 (switch_off ?maissa_sof_i ?maissa_ac_s))
    (task1 (switch_on ?maissa_ac_i ?maissa_ac_s))
    (task2 (auto_calibrate ?maissa_ac_s ?maissa_ac_i))))
 (:method method5
  :parameters ( ?maisa_ac_i - instrument ?maisa_ac_s - satellite)
  :task (activate_instrument ?maisa_ac_s ?maisa_ac_i)
  :ordered-subtasks (and
    (task0 (switch_on ?maisa_ac_i ?maisa_ac_s))
    (task1 (auto_calibrate ?maisa_ac_s ?maisa_ac_i))))
 (:method method6
  :parameters ( ?mactc_c_d - calib_direction ?mactc_c_i - instrument ?mactc_c_s - satellite ?mactc_tt_d_prev - direction)
  :task (auto_calibrate ?mactc_c_s ?mactc_c_i)
  :ordered-subtasks (and
    (task0 (turn_to ?mactc_c_s ?mactc_c_d ?mactc_tt_d_prev))
    (task1 (calibrate ?mactc_c_s ?mactc_c_i ?mactc_c_d))))
 (:method method7
  :parameters ( ?macc_c_d - calib_direction ?macc_c_i - instrument ?macc_c_s - satellite)
  :task (auto_calibrate ?macc_c_s ?macc_c_i)
  :ordered-subtasks (and
    (task0 (calibrate ?macc_c_s ?macc_c_i ?macc_c_d))))
 (:action turn_to
  :parameters ( ?t_s - satellite ?t_d_new - direction ?t_d_prev - direction)
  :precondition (and (pointing ?t_s ?t_d_prev))
  :effect (and (pointing ?t_s ?t_d_new) (not (pointing ?t_s ?t_d_prev))))
 (:action switch_on
  :parameters ( ?so_i - instrument ?so_s - satellite)
  :precondition (and (on_board ?so_i ?so_s) (power_avail ?so_s))
  :effect (and (power_on ?so_i) (not (calibrated ?so_i)) (not (power_avail ?so_s))))
 (:action switch_off
  :parameters ( ?sof_i - instrument ?sof_s - satellite)
  :precondition (and (on_board ?sof_i ?sof_s) (power_on ?sof_i))
  :effect (and (not (power_on ?sof_i)) (power_avail ?sof_s)))
 (:action calibrate
  :parameters ( ?c_s - satellite ?c_i - instrument ?c_d - calib_direction)
  :precondition (and (on_board ?c_i ?c_s) (calibration_target ?c_i ?c_d) (pointing ?c_s ?c_d) (power_on ?c_i))
  :effect (and (calibrated ?c_i)))
 (:action take_image
  :parameters ( ?ti_s - satellite ?ti_d - image_direction ?ti_i - instrument ?ti_m - mode)
  :precondition (and (calibrated ?ti_i) (pointing ?ti_s ?ti_d) (on_board ?ti_i ?ti_s) (power_on ?ti_i) (supports ?ti_i ?ti_m))
  :effect (and (have_image ?ti_d ?ti_m)))
)
